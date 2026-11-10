use clap::Parser;
use roboat::bedev2::AssetCreator;
use roboat::catalog::AssetType;
use roboat::ClientBuilder;

/// This example is made for programs that would restore old roblox games
/// Whenever you download a .rblx file and it has audios none of them will work.
/// This is because the audios isn't owned by you.
/// Reuploading them will succesfully restore the audios under a new ID.
///
/// Only an API key is needed. It needs the `asset:read`, `asset:write`
/// and `legacy-asset:manage` scopes.
#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    api_key: String,
    /// The user id that owns the API key, the audio is uploaded under this user.
    #[arg(long)]
    user_id: u64,
    #[arg(long)]
    asset_id: u64,
    /// Any place owned by the creator of the audio, needed if you don't own it.
    #[arg(long)]
    place_id: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let client = ClientBuilder::new().api_key(args.api_key).build();

    let existing_audio = client
        .fetch_cloud_asset_data(args.asset_id, args.place_id)
        .await?;

    let response = client
        .upload_asset(
            AssetCreator::User(args.user_id),
            "roboatTest".to_string(),
            "This is a roboat example".to_string(),
            AssetType::Audio,
            &existing_audio,
        )
        .await?;

    println!("Uploaded Audio!");

    loop {
        let operation = client.poll_upload_operation(response.path.clone()).await?;

        if operation.done {
            if let Some(resp) = operation.response {
                println!("New Asset ID: {:?}", resp.asset_id);
            }

            if let Some(error) = operation.error {
                println!("Upload failed: {}", error.message);
            }

            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Ok(())
}
