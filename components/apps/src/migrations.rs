#![allow(unused)]

use admin_app::migrations::Migrator;
use littlefs2::path;

pub(crate) const MIGRATION_VERSION_SPACE_EFFICIENCY: u32 = 1;

#[cfg(feature = "backend-auth")]
pub(crate) const TRUSSED_AUTH_FS_LAYOUT: trussed_auth::FilesystemLayout =
    trussed_auth::FilesystemLayout::V0;
#[cfg(feature = "se050")]
pub(crate) const SE050_BACKEND_FS_LAYOUT: trussed_se050_backend::FilesystemLayout =
    trussed_se050_backend::FilesystemLayout::V0;

/// TODO: When enabling the filesystem layout V1, fido-authenticator will also need to be bump and have its migration enabled
const _: () = {
    #[cfg(feature = "backend-auth")]
    assert!(matches!(
        TRUSSED_AUTH_FS_LAYOUT,
        trussed_auth::FilesystemLayout::V0
    ));
    #[cfg(feature = "se050")]
    assert!(matches!(
        SE050_BACKEND_FS_LAYOUT,
        trussed_se050_backend::FilesystemLayout::V0
    ));
    assert!(MIGRATORS.is_empty());
};

pub(crate) const MIGRATORS: &[Migrator] = &[];

// TODO: use when enabling migrations of trussed-auth and se050-backend and of fido-authenticator
const _MIGRATORS: &[Migrator] = &[
    // We first migrate the SE050 since this migration deletes data to make sure that the other
    // migrations succeed even on low block availability
    #[cfg(feature = "se050")]
    Migrator {
        migrate: |ifs, _efs| {
            trussed_se050_backend::migrate::migrate_remove_all_dat(ifs, &[path!("/opcard")])
        },
        version: MIGRATION_VERSION_SPACE_EFFICIENCY,
    },
    #[cfg(feature = "backend-auth")]
    Migrator {
        migrate: |ifs, _efs| {
            trussed_auth::migrate::migrate_remove_dat(
                ifs,
                &[
                    path!("opcard"),
                    path!("webcrypt"),
                    path!("secrets"),
                    path!("piv"),
                ],
            )
        },
        version: MIGRATION_VERSION_SPACE_EFFICIENCY,
    },
    Migrator {
        // FIDO migration
        migrate: |_ifs, _efs| todo!("Add fido migration"),
        version: MIGRATION_VERSION_SPACE_EFFICIENCY,
    },
];
