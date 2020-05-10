use structopt::StructOpt;

mod archive;
mod build;
mod client;
mod cmdline;
mod entrypoint;
mod logging;
mod gcloud;
mod nmkup;
mod nmkpkg;

type BoxError = Box<dyn std::error::Error>;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = cmdline::Opt::from_args();
    logging::setup(opt.debug);

    let nmk_dir = nmkup::find_nmkdir();
    // Installation should be done in order
    archive::install_or_update(&opt, &nmk_dir).await?;
    entrypoint::install(&nmk_dir).await?;
    nmkup::self_setup(&nmk_dir);
//    nmkpkg::install(&nmk_dir).await?;
    Ok(())
}