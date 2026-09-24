#![no_std]
#![no_main]

delog::generate_macros!();

use cortex_m_rt::{exception, ExceptionFrame};

// The boot ROM leaves MSPLIM inside our stack range, so lower it before Reset pushes anything.
core::arch::global_asm!(
    ".section .text.__pre_init, \"ax\"",
    ".global __pre_init",
    ".type __pre_init, %function",
    ".thumb_func",
    "__pre_init:",
    "    movw r0, #:lower16:_stack_end",
    "    movt r0, #:upper16:_stack_end",
    "    msr MSPLIM, r0",
    "    bx lr",
);

#[rtic::app(device = stm32n657_hal::pac, peripherals = true, dispatchers = [LPTIM1, LPTIM2, LPTIM3])]
mod app {
    use apdu_dispatch::{dispatch::ApduDispatch, interchanges::Channel as CcidChannel};
    use apps::Endpoints;
    use boards::{
        init::{CtaphidDispatch, Resources, UsbClasses},
        nkso3::{self, Storage, StorageChannel, UsbStorage, NKSO3},
        runtime,
        soc::{
            self,
            monotonic::SystickMonotonic,
            stm32n6::{self, mmc::Mmc},
        },
        store, Apps, Trussed,
    };
    use embedded_runner_lib::{VERSION, VERSION_STRING};
    use embedded_time::duration::Milliseconds;
    use interchange::Channel;
    use stm32n657_hal::{
        mmc::CardKind,
        pac::Interrupt,
        pwr::Pwr,
        rcc::Rcc,
        rng::Rng,
        timer::{MillisecondsCounter, Tim6},
    };
    use systick_monotonic::Systick;

    type Board = NKSO3;
    type Soc = <Board as boards::Board>::Soc;

    const USB_INTERRUPT: Interrupt = Interrupt::OTG1;

    #[shared]
    struct SharedResources {
        trussed: Trussed<Board>,
        apps: Apps<Board>,
        apdu_dispatch: ApduDispatch<'static>,
        ctaphid_dispatch: CtaphidDispatch<'static, 'static>,
        usb_classes: Option<UsbClasses<Soc>>,
        usb_storage: Option<UsbStorage<'static, <Soc as soc::Soc>::UsbBus>>,
        usb_timer: Option<MillisecondsCounter<Tim6>>,
        _mmc: Mmc,
    }

    #[local]
    struct LocalResources {
        endpoints: Endpoints,
    }

    #[monotonic(binds = SysTick, default = true)]
    type Monotonic = SystickMonotonic;

    #[init(local = [resources: Resources<NKSO3> = Resources::new()])]
    fn init(ctx: init::Context) -> (SharedResources, LocalResources, init::Monotonics) {
        let mut init_status = apps::InitStatus::default();

        #[cfg(feature = "alloc")]
        embedded_runner_lib::init_alloc();

        boards::init::init_logger::<Board>(VERSION_STRING);

        let soc = stm32n6::init_bootup(ctx.device.BSEC);
        let rcc = Rcc::new(ctx.device.RCC);
        let clock_config = rcc.clock_config();

        let (board_gpio, mmc) = nkso3::init_pins(
            ctx.device.GPIOC_S,
            ctx.device.GPIOG_S,
            ctx.device.SDMMC2_S,
            &rcc,
        );

        let pwr = Pwr::new(ctx.device.PWR_S);
        pwr.enable_mmc_vddio();

        let mut mmc = mmc.enable(&rcc, CardKind::Sd).unwrap();

        #[cfg(feature = "sdmmc-tests")]
        {
            let mut block_read = [[1u8; 512]; 1];
            let block = [[2u8; 512]; 1];
            mmc.write_blocks(&block, 0).unwrap();
            mmc.read_blocks(&mut block_read, 0).unwrap();
            assert_eq!(block_read, block);
        }

        let usb_bus = stm32n6::setup_usb_bus(
            &mut ctx.local.resources.board,
            ctx.device.OTG1_S,
            &pwr,
            &rcc,
            clock_config,
        );

        let (internal_storage, external_storage) = nkso3::init_storage();
        let store = store::init_store(
            &mut ctx.local.resources.store,
            internal_storage,
            external_storage,
            false,
            &mut init_status,
        );

        static STORAGE_CHANNEL: StorageChannel = Channel::new();
        static NFC_CHANNEL: CcidChannel = Channel::new();
        let (storage_rq, storage_rp) = STORAGE_CHANNEL.split().unwrap();
        let (_nfc_rq, nfc_rp) = NFC_CHANNEL.split().unwrap();
        let usb_nfc = embedded_runner_lib::init_usb_nfc(
            &mut ctx.local.resources.usb,
            Some(usb_bus),
            None,
            nfc_rp,
            storage_rp,
        );

        let user_interface = nkso3::init_ui(board_gpio, ctx.device.TIM7_S, &rcc, clock_config);

        let mut dev_rng = Rng::new(ctx.device.RNG_S, &rcc);
        let mut trussed = boards::init::init_trussed(
            &mut dev_rng,
            store,
            user_interface,
            &mut init_status,
            None,
            #[cfg(feature = "se050")]
            None,
        );

        let (apps, endpoints) = boards::init::init_apps(
            &soc,
            &mut trussed,
            init_status,
            &store,
            false,
            VERSION,
            VERSION_STRING,
            Storage::new(storage_rq),
        );

        let usb_timer = if usb_nfc.usb_storage.is_some() {
            let tim6 = Tim6::new(ctx.device.TIM6_S, &rcc);
            Some(MillisecondsCounter::new(tim6, clock_config))
        } else {
            None
        };
        let systick = Systick::new(ctx.core.SYST, clock_config.sys_bus_ck().to_Hz());

        ui::spawn_after(Milliseconds(2500)).ok();

        (
            SharedResources {
                trussed,
                apps,
                apdu_dispatch: usb_nfc.apdu_dispatch,
                ctaphid_dispatch: usb_nfc.ctaphid_dispatch,
                usb_classes: usb_nfc.usb_classes,
                usb_storage: usb_nfc.usb_storage,
                usb_timer,
                _mmc: mmc,
            },
            LocalResources { endpoints },
            init::Monotonics(systick.into()),
        )
    }

    #[idle(shared = [apps, apdu_dispatch, ctaphid_dispatch, usb_classes, usb_storage, usb_timer])]
    fn idle(ctx: idle::Context) -> ! {
        let idle::SharedResources {
            mut apps,
            mut apdu_dispatch,
            mut ctaphid_dispatch,
            mut usb_classes,
            mut usb_storage,
            mut usb_timer,
        } = ctx.shared;

        trace!("idle");

        loop {
            #[cfg(not(feature = "no-delog"))]
            boards::init::Delogger::flush();

            let (usb_activity, _nfc_activity) =
                (&mut apps, &mut apdu_dispatch, &mut ctaphid_dispatch)
                    .lock(|apps, apdu, ctaphid| runtime::poll_dispatchers(apdu, ctaphid, apps));
            if usb_activity {
                rtic::pend(USB_INTERRUPT);
            }

            usb_classes.lock(|usb_classes| {
                usb_storage.lock(|usb_storage| {
                    usb_timer.lock(|usb_timer| {
                        runtime::poll_usb(
                            usb_classes,
                            usb_storage,
                            ccid_keepalive::spawn_after,
                            ctaphid_keepalive::spawn_after,
                            monotonics::now(),
                            move |device| device.bus().force_reset(usb_timer.as_mut().unwrap()),
                        );
                    });
                });
            });
        }
    }

    #[task(priority = 2, binds = LPTIM4, shared = [trussed], local = [endpoints])]
    fn task_trussed(ctx: task_trussed::Context) {
        let mut trussed = ctx.shared.trussed;

        trussed.lock(|trussed| {
            runtime::run_trussed(trussed, ctx.local.endpoints);
        });
    }

    #[task(priority = 3, binds = OTG1, shared = [usb_classes, usb_storage, usb_timer])]
    fn task_usb(ctx: task_usb::Context) {
        let mut usb_classes = ctx.shared.usb_classes;
        let mut usb_storage = ctx.shared.usb_storage;
        let mut usb_timer = ctx.shared.usb_timer;

        usb_classes.lock(|usb_classes| {
            usb_storage.lock(|usb_storage| {
                usb_timer.lock(|usb_timer| {
                    runtime::poll_usb(
                        usb_classes,
                        usb_storage,
                        ccid_keepalive::spawn_after,
                        ctaphid_keepalive::spawn_after,
                        monotonics::now(),
                        move |device| device.bus().force_reset(usb_timer.as_mut().unwrap()),
                    );
                });
            });
        });
    }

    #[task(priority = 3, shared = [usb_classes])]
    fn ccid_keepalive(ctx: ccid_keepalive::Context) {
        let mut usb_classes = ctx.shared.usb_classes;

        usb_classes.lock(|usb_classes| {
            runtime::ccid_keepalive(usb_classes, ccid_keepalive::spawn_after);
        });
    }

    #[task(priority = 3, shared = [usb_classes])]
    fn ctaphid_keepalive(ctx: ctaphid_keepalive::Context) {
        let mut usb_classes = ctx.shared.usb_classes;

        usb_classes.lock(|usb_classes| {
            runtime::ctaphid_keepalive(usb_classes, ctaphid_keepalive::spawn_after);
        });
    }

    #[task(priority = 1, shared = [trussed])]
    fn ui(ctx: ui::Context) {
        let mut trussed = ctx.shared.trussed;

        trussed.lock(|trussed| {
            trussed.update_ui();
        });
        ui::spawn_after(Milliseconds(125)).ok();
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    boards::handle_panic::<boards::nkso3::NKSO3>(info)
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    boards::handle_hard_fault::<boards::nkso3::NKSO3>(ef)
}
