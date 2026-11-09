use clap::Parser;
use roboat::catalog::AssetType;
use roboat::ClientBuilder;

/// This example is made for programs that would restore old roblox games
/// Whenever you download a .rblx file and it has audios none of them will work.
/// This is because the audio isn't owned by you.
/// Reuploading them will succesfully restore the audios under a new ID.
///
#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    api_key: String,
    #[arg(long, short)]
    roblosecurity: String,
    #[arg(long)]
    asset_id: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let client = ClientBuilder::new()
        .roblosecurity(args.roblosecurity)
        .build();

    let asset_id = args.asset_id;
    let existing_audio = client.fetch_asset_data(asset_id).await?;

    let response = client
        .upload_asset(
            args.api_key.clone(),
            None,
            "roboatTest".to_string(),
            "This is a roboat example".to_string(),
            AssetType::Audio,
            &existing_audio,
        )
        .await?;

    println!("Uploaded Audio!");

    loop {
        let operation = client
            .poll_upload_operation(args.api_key.clone(), response.path.clone())
            .await?;

        if operation.done {
            if let Some(resp) = operation.response {
                println!("New Asset ID: {:?}", resp.asset_id);
            }

            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Ok(())
}
