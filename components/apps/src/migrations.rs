#![allow(unused)]

use admin_app::migrations::Migrator;
use littlefs2_core::path;

pub(crate) const MIGRATION_VERSION_SPACE_EFFICIENCY: u32 = 1;

#[cfg(feature = "backend-auth")]
pub(crate) const TRUSSED_AUTH_FS_LAYOUT: trussed_auth::FilesystemLayout =
    trussed_auth::FilesystemLayout::V1;
#[cfg(feature = "se050")]
pub(crate) const SE050_BACKEND_FS_LAYOUT: trussed_se050_backend::FilesystemLayout =
    trussed_se050_backend::FilesystemLayout::V1;

pub(crate) const MIGRATORS: &[Migrator] = &[
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
];
