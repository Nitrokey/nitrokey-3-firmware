use core::{marker::PhantomData, mem::MaybeUninit};

use apps::InitStatus;
use littlefs2::{
    const_ram_storage,
    driver::Storage,
    fs::{Allocation, Filesystem},
    io::Result,
    object_safe::DynFilesystem,
};
use trussed::store::Store;
use trussed_manage::FACTORY_RESET_MARKER_FILE;

use crate::Board;

// 8KB of RAM
const_ram_storage!(
    name = VolatileStorage,
    erase_value = 0xff,
    read_size = 16,
    write_size = 256,
    cache_size = 256,
    // We use 256 instead of the default 512 to avoid loosing too much space to nearly empty blocks containing only folder metadata.
    block_size = 256,
    block_count = 8192 / 256,
    lookahead_size = 1,
);

pub struct StoreResources<B: Board> {
    internal: StorageResources<B::InternalStorage>,
    external: StorageResources<B::ExternalStorage>,
    volatile: StorageResources<VolatileStorage>,
}

impl<B: Board> StoreResources<B> {
    pub const fn new() -> Self {
        Self {
            internal: StorageResources::new(),
            external: StorageResources::new(),
            volatile: StorageResources::new(),
        }
    }
}

pub struct StorageResources<S: Storage + 'static> {
    fs: MaybeUninit<Filesystem<'static, S>>,
    alloc: MaybeUninit<Allocation<S>>,
    storage: MaybeUninit<S>,
}

impl<S: Storage + 'static> StorageResources<S> {
    pub const fn new() -> Self {
        Self {
            fs: MaybeUninit::uninit(),
            alloc: MaybeUninit::uninit(),
            storage: MaybeUninit::uninit(),
        }
    }
}

struct StorePointers {
    ifs: MaybeUninit<&'static dyn DynFilesystem>,
    efs: MaybeUninit<&'static dyn DynFilesystem>,
    vfs: MaybeUninit<&'static dyn DynFilesystem>,
}

impl StorePointers {
    const fn new() -> Self {
        Self {
            ifs: MaybeUninit::uninit(),
            efs: MaybeUninit::uninit(),
            vfs: MaybeUninit::uninit(),
        }
    }
}

pub struct RunnerStore<B> {
    _marker: PhantomData<*mut B>,
}

impl<B: Board> RunnerStore<B> {
    fn new(
        ifs: &'static dyn DynFilesystem,
        efs: &'static dyn DynFilesystem,
        vfs: &'static dyn DynFilesystem,
    ) -> Self {
        unsafe {
            let pointers = Self::pointers();
            pointers.ifs.write(ifs);
            pointers.efs.write(efs);
            pointers.vfs.write(vfs);
        }

        Self {
            _marker: Default::default(),
        }
    }

    unsafe fn pointers() -> &'static mut StorePointers {
        static mut POINTERS: StorePointers = StorePointers::new();
        (&raw mut POINTERS).as_mut().unwrap()
    }
}

impl<B> Clone for RunnerStore<B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<B> Copy for RunnerStore<B> {}

impl<B: Board> Store for RunnerStore<B> {
    fn ifs(&self) -> &dyn DynFilesystem {
        unsafe { Self::pointers().ifs.assume_init() }
    }

    fn efs(&self) -> &dyn DynFilesystem {
        unsafe { Self::pointers().efs.assume_init() }
    }

    fn vfs(&self) -> &dyn DynFilesystem {
        unsafe { Self::pointers().vfs.assume_init() }
    }
}

pub fn init_store<B: Board>(
    resources: &'static mut StoreResources<B>,
    int_flash: B::InternalStorage,
    ext_flash: B::ExternalStorage,
    simulated_efs: bool,
    status: &mut InitStatus,
) -> RunnerStore<B> {
    let ifs_storage = resources.internal.storage.write(int_flash);
    let ifs_alloc = resources
        .internal
        .alloc
        .write(Filesystem::allocate(&ifs_storage));
    let efs_storage = resources.external.storage.write(ext_flash);
    let efs_alloc = resources
        .external
        .alloc
        .write(Filesystem::allocate(&efs_storage));
    let vfs_storage = resources.volatile.storage.write(VolatileStorage::new());
    let vfs_alloc = resources
        .volatile
        .alloc
        .write(Filesystem::allocate(&vfs_storage));

    let ifs = match init_ifs::<B>(ifs_storage, ifs_alloc, efs_storage, status) {
        Ok(ifs) => resources.internal.fs.write(ifs),
        Err(_e) => {
            error!("IFS Mount Error {:?}", _e);
            panic!("IFS");
        }
    };

    let efs = match init_efs::<B>(efs_storage, efs_alloc, simulated_efs, status) {
        Ok(efs) => resources.external.fs.write(efs),
        Err(_e) => {
            error!("EFS Mount Error {:?}", _e);
            panic!("EFS");
        }
    };

    let vfs = match init_vfs(vfs_storage, vfs_alloc) {
        Ok(vfs) => resources.volatile.fs.write(vfs),
        Err(_e) => {
            error!("VFS Mount Error {:?}", _e);
            panic!("VFS");
        }
    };

    RunnerStore::new(ifs, efs, vfs)
}

#[inline(always)]
fn init_ifs<B: Board>(
    ifs_storage: &'static mut B::InternalStorage,
    ifs_alloc: &'static mut Allocation<B::InternalStorage>,
    efs_storage: &mut B::ExternalStorage,
    status: &mut InitStatus,
) -> Result<Filesystem<'static, B::InternalStorage>> {
    if cfg!(feature = "format-filesystem") {
        Filesystem::format(ifs_storage).ok();
    }

    if !Filesystem::is_mountable(ifs_storage) {
        // handle provisioner
        if cfg!(feature = "provisioner") {
            info_now!("IFS mount failed - provisioner => formatting");
            Filesystem::format(ifs_storage).ok();
        } else {
            status.insert(InitStatus::INTERNAL_FLASH_ERROR);
            error_now!("IFS mount-fail");
            B::recover_ifs(ifs_storage, ifs_alloc, efs_storage).ok();
        }
    }

    B::prepare_ifs(ifs_storage);

    Filesystem::mount(ifs_alloc, ifs_storage)
}

#[inline(always)]
fn init_efs<'a, B: Board>(
    efs_storage: &'a mut B::ExternalStorage,
    efs_alloc: &'a mut Allocation<B::ExternalStorage>,
    simulated_efs: bool,
    status: &mut InitStatus,
) -> Result<Filesystem<'a, B::ExternalStorage>> {
    use littlefs2::fs::{Config as LfsConfig, MountFlags};
    if cfg!(feature = "format-filesystem") {
        Filesystem::format(efs_storage).ok();
    }

    let fs = Filesystem::mount_or_else(efs_alloc, efs_storage, |err, storage, efs_alloc| {
        let mut config = LfsConfig::default();
        error_now!("EFS Mount Error {:?}", err);

        // Maybe the case is that the block count is wrong
        if err == littlefs2::io::Error::INVALID {
            config.mount_flags = MountFlags::DISABLE_BLOCK_COUNT_CHECK;

            let mut mounted_with_wrong_block_count = false;

            let shrink_res =
                Filesystem::mount_and_then_with_config(storage, config.clone(), |fs| {
                    mounted_with_wrong_block_count = true;
                    fs.shrink(fs.total_blocks())
                });
            match shrink_res {
                Ok(_) => return Ok(()),
                // The error is just the block count and shrinking failed, we warn that reformat is required
                Err(_) if mounted_with_wrong_block_count => {
                    status.insert(InitStatus::EXT_FLASH_NEED_REFORMAT);
                    *efs_alloc = Allocation::with_config(storage, config);
                    return Ok(());
                }
                // Failed to mount when ignoring block count check. Error is something else
                // Go to normal flow
                Err(_) => {}
            }
        }

        let fmt_ext = Filesystem::format(storage);
        if simulated_efs && fmt_ext == Err(littlefs2::io::Error::NO_SPACE) {
            info_now!("Formatting simulated EFS failed as expected");
        } else {
            error_now!("EFS Reformat {:?}", fmt_ext);
            status.insert(InitStatus::EXTERNAL_FLASH_ERROR);
        }
        Ok(())
    })?;

    if fs.exists(FACTORY_RESET_MARKER_FILE) {
        debug_now!("Reformatting EFS filesystem");
        let (efs_alloc, efs_storage) = fs.into_inner();
        Filesystem::format(efs_storage).ok();
        Filesystem::mount(efs_alloc, efs_storage)
    } else {
        Ok(fs)
    }
}

#[inline(always)]
fn init_vfs(
    vfs_storage: &'static mut VolatileStorage,
    vfs_alloc: &'static mut Allocation<VolatileStorage>,
) -> Result<Filesystem<'static, VolatileStorage>> {
    if !Filesystem::is_mountable(vfs_storage) {
        Filesystem::format(vfs_storage).ok();
    }
    Filesystem::mount(vfs_alloc, vfs_storage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        soc::Soc,
        ui::{buttons::UserPresence, rgb_led::RgbLed, Clock},
    };
    use apps::Reboot;
    use cortex_m::interrupt::InterruptNumber;
    use embedded_time::duration::Milliseconds;
    use littlefs2::{path, path::PathBuf};
    use nfc_device::traits::nfc::{Device as NfcDevice, Error as NfcError, State as NfcState};
    use usb_device::bus::UsbBus;

    struct TestBoard<EfsStorage> {
        __: PhantomData<EfsStorage>,
    }
    struct TestSoc;
    struct DummyUsbBus;
    struct DummyClock;
    #[derive(Clone, Copy)]
    struct DummyInterrupt;
    pub struct DummyNfc;
    pub struct DummyButtons;
    pub struct DummyLed;
    impl UserPresence for DummyButtons {
        fn check_user_presence(&mut self) -> trussed::types::consent::Level {
            unimplemented!()
        }
    }
    impl RgbLed for DummyLed {
        fn set_panic_led() {
            unimplemented!()
        }

        fn red(&mut self, _intensity: u8) {
            unimplemented!()
        }

        fn green(&mut self, _intensity: u8) {
            unimplemented!()
        }

        fn blue(&mut self, _intensity: u8) {
            unimplemented!()
        }
    }

    impl NfcDevice for DummyNfc {
        fn read(&mut self, _buf: &mut [u8]) -> Result<NfcState, NfcError> {
            Err(NfcError::NoActivity)
        }
        fn send(&mut self, _buf: &[u8]) -> Result<(), NfcError> {
            Err(NfcError::NoActivity)
        }
        fn frame_size(&self) -> usize {
            0
        }
    }

    unsafe impl InterruptNumber for DummyInterrupt {
        fn number(self) -> u16 {
            unimplemented!()
        }
    }
    impl Clock for DummyClock {
        fn uptime(&mut self) -> core::time::Duration {
            unimplemented!()
        }
    }
    impl UsbBus for DummyUsbBus {
        fn alloc_ep(
            &mut self,
            _ep_dir: usb_device::UsbDirection,
            _ep_addr: Option<usb_device::endpoint::EndpointAddress>,
            _ep_type: usb_device::endpoint::EndpointType,
            _max_packet_size: u16,
            _interval: u8,
        ) -> usb_device::Result<usb_device::endpoint::EndpointAddress> {
            unimplemented!()
        }
        fn enable(&mut self) {
            unimplemented!()
        }

        fn reset(&self) {
            unimplemented!()
        }

        fn set_device_address(&self, _addr: u8) {
            unimplemented!()
        }

        fn write(
            &self,
            _ep_addr: usb_device::endpoint::EndpointAddress,
            _buf: &[u8],
        ) -> usb_device::Result<usize> {
            unimplemented!()
        }

        fn read(
            &self,
            _ep_addr: usb_device::endpoint::EndpointAddress,
            _buf: &mut [u8],
        ) -> usb_device::Result<usize> {
            unimplemented!()
        }

        fn set_stalled(&self, _ep_addr: usb_device::endpoint::EndpointAddress, _stalled: bool) {
            unimplemented!()
        }

        fn is_stalled(&self, _ep_addr: usb_device::endpoint::EndpointAddress) -> bool {
            unimplemented!()
        }

        fn suspend(&self) {
            unimplemented!()
        }

        fn resume(&self) {
            unimplemented!()
        }

        fn poll(&self) -> usb_device::bus::PollResult {
            unimplemented!()
        }
    }
    impl Soc for TestSoc {
        type UsbBus = DummyUsbBus;
        type Clock = DummyClock;
        type Duration = Milliseconds;
        type Interrupt = DummyInterrupt;

        const SYSCALL_IRQ: Self::Interrupt = DummyInterrupt;

        const SOC_NAME: &'static str = "Dummy SOC";

        const VARIANT: apps::Variant = apps::Variant::Usbip;

        fn uuid(&self) -> &crate::soc::Uuid {
            todo!()
        }
    }

    impl Reboot for TestSoc {
        fn reboot() -> ! {
            unimplemented!()
        }

        fn reboot_to_firmware_update() -> ! {
            unimplemented!()
        }

        fn reboot_to_firmware_update_destructive() -> ! {
            unimplemented!()
        }

        fn locked() -> bool {
            unimplemented!()
        }
    }

    const FULL_EXTERNAL_STORAGE_BLOCK_COUNT: usize = 0x20_0000 / 4096;
    const CROPPED_EXTERNAL_STORAGE_BLOCK_COUNT: usize = FULL_EXTERNAL_STORAGE_BLOCK_COUNT - 32;

    const_ram_storage!(
        name = ExternalStorageFull,
        erase_value = 0xFF,
        read_size = 4,
        write_size = 256,
        cache_size_ty = littlefs2::consts::U256,
        block_size = 4096,
        block_count = FULL_EXTERNAL_STORAGE_BLOCK_COUNT,
        lookahead_size_ty = littlefs2::consts::U1,
        filename_max_plus_one_ty = littlefs2::consts::U1,
        path_max_plus_one_ty = littlefs2::consts::U2,
    );

    const_ram_storage!(
        name = ExternalStorageCropped,
        erase_value = 0xFF,
        read_size = 4,
        write_size = 256,
        cache_size_ty = littlefs2::consts::U256,
        block_size = 4096,
        block_count = CROPPED_EXTERNAL_STORAGE_BLOCK_COUNT,
        lookahead_size_ty = littlefs2::consts::U1,
        filename_max_plus_one_ty = littlefs2::consts::U1,
        path_max_plus_one_ty = littlefs2::consts::U2,
    );

    const_ram_storage!(
        name = InternalStorage,
        erase_value = 0xFF,
        read_size = 4,
        write_size = 256,
        cache_size_ty = littlefs2::consts::U256,
        block_size = 4096,
        block_count = 0x20_0000 / 4096,
        lookahead_size_ty = littlefs2::consts::U1,
        filename_max_plus_one_ty = littlefs2::consts::U1,
        path_max_plus_one_ty = littlefs2::consts::U2,
    );

    impl<EfsStorage: Storage + 'static> Board for TestBoard<EfsStorage> {
        type Soc = TestSoc;

        type Resources = ();

        type NfcDevice = DummyNfc;

        type Buttons = DummyButtons;

        type Led = DummyLed;

        type InternalStorage = InternalStorage;

        type ExternalStorage = EfsStorage;

        type Se050Timer = ();

        type Twi = ();

        const BOARD_NAME: &'static str = "Dummy board";

        const HAS_NFC: bool = false;
    }

    #[test]
    fn test_init_efs_from_scratch() {
        let mut status = InitStatus::empty();
        const EFS_STORAGE_SIZE: usize = size_of::<ExternalStorageFull>();
        // Avoid stack allocation that overflows stack in dev profile
        let mut efs_storage_full: Box<[u8; EFS_STORAGE_SIZE]> = vec![0; EFS_STORAGE_SIZE]
            .into_boxed_slice()
            .try_into()
            .unwrap();
        let efs_storage_full: &mut ExternalStorageFull =
            unsafe { &mut *(&raw mut *efs_storage_full as *mut ExternalStorageFull) };
        let mut efs_storage_cropped: Box<[u8; EFS_STORAGE_SIZE]> = vec![0; EFS_STORAGE_SIZE]
            .into_boxed_slice()
            .try_into()
            .unwrap();
        let efs_storage_cropped: &mut ExternalStorageCropped =
            unsafe { &mut *(&raw mut *efs_storage_cropped as *mut ExternalStorageCropped) };
        let efs_alloc = &mut Allocation::new();
        let efs_alloc_cropped = &mut Allocation::new();
        let storage = init_efs::<TestBoard<ExternalStorageFull>>(
            efs_storage_full,
            efs_alloc,
            false,
            &mut status,
        )
        .unwrap();
        // The flash was formatted since it's the first boot;
        assert_eq!(status, InitStatus::EXTERNAL_FLASH_ERROR);
        status = InitStatus::empty();

        // Check that first mount is empty
        storage
            .read_dir_and_then(path!("/"), |dir| {
                assert!(dir.next().is_some());
                assert!(dir.next().is_some());
                assert!(dir.next().is_none());
                Ok(())
            })
            .unwrap();

        // Check that factory reset correctly reformats the filesystem
        storage.write(FACTORY_RESET_MARKER_FILE, &[]).unwrap();
        let (_, efs_storage_full) = storage.into_inner();
        let efs_alloc = &mut Allocation::new();

        let storage = init_efs::<TestBoard<ExternalStorageFull>>(
            efs_storage_full,
            efs_alloc,
            false,
            &mut status,
        )
        .unwrap();
        storage
            .read_dir_and_then(path!("/"), |dir| {
                assert!(dir.next().is_some());
                assert!(dir.next().is_some());
                assert!(dir.next().is_none());
                Ok(())
            })
            .unwrap();
        assert_eq!(storage.total_blocks(), 0x20_0000 / 4096);

        let (_, efs_storage) = storage.into_inner();
        let efs_alloc = &mut Allocation::new();
        let storage =
            init_efs::<TestBoard<ExternalStorageFull>>(efs_storage, efs_alloc, false, &mut status)
                .unwrap();
        assert_eq!(status, InitStatus::empty());
        let mut data = vec![0xEE; 4096];
        for i in 0.. {
            data.fill(i as u8);
            let filename = format!("file-{i}");
            match storage.write(&PathBuf::try_from(&*filename).unwrap(), &data) {
                Ok(_) => continue,
                Err(littlefs2::io::Error::NO_SPACE) => {
                    break;
                }
                Err(_err) => panic!("Unexpected error: {_err:?}"),
            }
        }
        let mut data = vec![0xEE; 256];
        for i in 0.. {
            data.fill(i as u8);
            let filename = format!("file-small-{i}");
            match storage.write(&PathBuf::try_from(&*filename).unwrap(), &data) {
                Ok(_) => continue,
                Err(littlefs2::io::Error::NO_SPACE) => {
                    break;
                }
                Err(_err) => panic!("Unexpected error: {_err:?}"),
            }
        }
        let (_, efs_storage_full) = storage.into_inner();
        let efs_alloc = &mut Allocation::new();

        efs_storage_cropped
            .buf
            .copy_from_slice(&efs_storage_full.buf[..CROPPED_EXTERNAL_STORAGE_BLOCK_COUNT * 4096]);

        status = InitStatus::empty();

        let storage = init_efs::<TestBoard<ExternalStorageCropped>>(
            efs_storage_cropped,
            efs_alloc_cropped,
            false,
            &mut status,
        )
        .unwrap();
        assert_eq!(status, InitStatus::EXT_FLASH_NEED_REFORMAT);
        let (_, efs_storage_cropped) = storage.into_inner();
        let efs_alloc_cropped = &mut Allocation::new();

        efs_storage_cropped.buf.fill(0);
        efs_storage_full.buf.fill(0);

        status = InitStatus::empty();
        let storage = init_efs::<TestBoard<ExternalStorageFull>>(
            efs_storage_full,
            efs_alloc,
            false,
            &mut status,
        )
        .unwrap();
        // Storage reformatted
        assert_eq!(status, InitStatus::EXTERNAL_FLASH_ERROR);
        status = InitStatus::empty();

        // Check that first mount is empty
        storage
            .read_dir_and_then(path!("/"), |dir| {
                assert!(dir.next().is_some());
                assert!(dir.next().is_some());
                assert!(dir.next().is_none());
                Ok(())
            })
            .unwrap();

        // Fill a bit but not too much
        let mut data = vec![0; 4096];
        for i in 0..10 {
            data.fill(i as u8);
            let filename = format!("file-{i}");
            match storage.write(&PathBuf::try_from(&*filename).unwrap(), &data) {
                Ok(_) => continue,
                Err(_err) => panic!("Unexpected error: {_err:?}"),
            }
        }
        let (_, efs_storage_full) = storage.into_inner();
        // let efs_alloc = &mut Allocation::new();
        efs_storage_cropped
            .buf
            .copy_from_slice(&efs_storage_full.buf[..CROPPED_EXTERNAL_STORAGE_BLOCK_COUNT * 4096]);

        let storage = init_efs::<TestBoard<ExternalStorageCropped>>(
            efs_storage_cropped,
            efs_alloc_cropped,
            false,
            &mut status,
        )
        .unwrap();
        assert_eq!(status, InitStatus::empty());
        // Check that all data is there
        let mut test_data = vec![0; 4096];
        for i in 0..10 {
            test_data.fill(i as u8);
            let filename = format!("file-{i}");
            let file = storage
                .read::<4096>(&PathBuf::try_from(&*filename).unwrap())
                .unwrap();
            assert_eq!(file, &*test_data);
        }
    }
}
