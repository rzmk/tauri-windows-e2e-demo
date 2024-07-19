use anyhow::Result;
use fantoccini::{Client, ClientBuilder, Locator};
use serde_json::json;
use std::time::Duration;
use tokio::{
    process::{Child, Command},
    time::sleep,
};

pub async fn spawn_tauri_driver() -> Result<Child> {
    println!("Starting tauri-driver...");
    Ok(Command::new("tauri-driver").spawn()?)
}

pub async fn spawn_webdriver_client() -> Result<Client> {
    let capabilities = json!({
        "tauri:options": {
            "application": "../src-tauri/target/release/tauri-windows-e2e-demo.exe",
            "webviewOptions": {}
        }
    });

    ClientBuilder::native()
        .capabilities(capabilities.as_object().unwrap().clone())
        .connect("http://localhost:4444")
        .await
        .map_err(anyhow::Error::msg)
}

pub async fn kill_drivers() -> Result<()> {
    println!("Killing any existing msedgedriver.exe and tauri-driver.exe processes...");
    Command::new("taskkill")
        .args(&["/F", "/IM", "tauri-driver.exe"])
        .output()
        .await?;
    Command::new("taskkill")
        .args(&["/F", "/IM", "msedgedriver.exe"])
        .output()
        .await?;
    Ok(())
}

async fn click_the_button(client: &Client) -> Result<()> {
    loop {
        sleep(Duration::from_secs(1)).await;
        let button_element = client.find(Locator::Id("clickable-button")).await;
        if let Ok(button) = button_element {
            println!("Found button element with ID 'clickable-button'.");
            println!("Clicking button.");
            button.click().await?;
            break;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Kill any running msedgedriver.exe and tauri-driver.exe processes
    kill_drivers().await?;

    // Spawn tauri-driver and the webdriver client
    let mut webdriver_process = spawn_tauri_driver().await?;
    let client = spawn_webdriver_client().await?;

    // Perform actions (click the button)
    click_the_button(&client).await?;

    // Wait a few seconds to see if the button was clicked
    println!("Check if the button was clicked! Closing in 5 seconds.");
    sleep(Duration::from_secs(5)).await;
    // Close the webdriver client and ensure that tauri-driver is killed
    client.close().await?;
    webdriver_process.kill().await?;
    // Finally clean up any remaining processes
    kill_drivers().await?;

    Ok(())
}
