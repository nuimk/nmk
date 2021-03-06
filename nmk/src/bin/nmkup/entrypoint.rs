use std::io;
use std::path::Path;

use bytes::{Buf, Bytes};

use nmk::bin_name::NMK;
use nmk::gcs::{download_file, get_object_meta, get_object_meta_url, ObjectMeta};
use nmk::home::NmkHome;
use nmk::setup::install;

use crate::build::Target;
use crate::cmdline::CmdOpt;

const TAG: &str = "entrypoint";

fn install_entrypoint(data: Bytes, dst: impl AsRef<Path>) -> io::Result<()> {
    let mut reader = xz2::read::XzDecoder::new(data.reader());
    install(&mut reader, dst)
}

const NMK_META: &str = ".nmk.meta";

#[derive(Copy, Clone)]
pub enum EntrypointInstallation {
    Installed,
    Up2Date,
}

pub async fn install_or_update(
    cmd_opt: &CmdOpt,
    nmk_home: &NmkHome,
) -> nmk::Result<EntrypointInstallation> {
    let target = Target::detect().expect("unsupported arch");
    let tar_file = target.remote_binary_name("nmk");
    let meta_path = nmk_home.as_path().join(NMK_META);
    let meta_url = get_object_meta_url(&tar_file);

    let client = reqwest::Client::new();

    log::debug!("{}: Getting metadata.", TAG);
    let meta = get_object_meta(&client, &meta_url).await?;
    log::debug!("{}: Received metadata.", TAG);
    let entrypoint_path = nmk_home.nmk_path().bin().join(NMK);
    if !cmd_opt.force && is_entrypoint_up2date(&meta_path, &meta, &entrypoint_path) {
        log::info!("{}: Already up to date.", TAG);
        Ok(EntrypointInstallation::Up2Date)
    } else {
        let data_url = &meta.media_link;
        log::debug!("{}: Getting data from {}.", TAG, data_url);
        let data = download_file(&client, data_url).await?;
        log::debug!("{}: Received data.", TAG);
        install_entrypoint(data, entrypoint_path)?;
        meta.write_to_file(&meta_path);
        log::info!("{}: Done.", TAG);
        Ok(EntrypointInstallation::Installed)
    }
}

fn is_entrypoint_up2date(meta_path: &Path, gcs_meta: &ObjectMeta, entrypoint_path: &Path) -> bool {
    if !entrypoint_path.exists() {
        return false;
    }
    if !meta_path.exists() {
        log::debug!("{}: Not found cached metadata.", TAG);
        return false;
    }

    let cached_meta = ObjectMeta::read_from_file(meta_path);
    log::debug!("{}: gcs generation {}.", TAG, gcs_meta.generation);
    log::debug!("{}: cached generation {}.", TAG, cached_meta.generation);
    cached_meta.generation == gcs_meta.generation
}
