use std::env::{self, VarError};
use std::path::PathBuf;

use anyhow::{Context, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use clap::Parser;

type OpenAIClient = async_openai::Client<OpenAIConfig>;

#[derive(Parser)]
struct HowToCli {
    #[arg(value_name = "ACTION")]
    /// The high-level action you would like to get a CLI command for.
    action: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = HowToCli::parse();

    let result = cli(args).await;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

async fn cli(args: HowToCli) -> Result<()> {
    let action = args.action;
    let openai = get_openai_client().await?;
    let command = generate_command(&openai, &action).await?;
    println!("{}", command);
    Ok(())
}

const SYSTEM_MESSAGE: &'static str = r#"
You are an expert Unix system operator. You have intimate and detailed knowledge of CLI tools, both old and new.

When the user asks for a command that accomplishes a high-level action, you respond with a CLI command that accomplishes that action.

Example input:
<action>
go to my home directory
</action>

Example output:
<command>
cd ~
</command>

If the action cannot be accomplished via the CLI, you must respond with:
<no_command/>
"#;

async fn generate_command(openai: &OpenAIClient, action: &str) -> Result<String> {
    let system_message: ChatCompletionRequestMessage =
        ChatCompletionRequestSystemMessageArgs::default()
            .content(SYSTEM_MESSAGE)
            .build()
            .expect("system message is valid")
            .into();

    let user_message: ChatCompletionRequestMessage =
        ChatCompletionRequestUserMessageArgs::default()
            .content(format!("<action>\n{}\n</action>", action.trim()))
            .build()
            .expect("user message is valid")
            .into();

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-2024-08-06")
        .max_tokens(1024u32)
        .temperature(0.0)
        .messages([system_message, user_message])
        .build()
        .expect("request is valid");

    let mut response = openai
        .chat()
        .create(request)
        .await
        .context("Unable to generate command. OpenAI request failed.")?;

    let choice = response
        .choices
        .pop()
        .context("Unable to generate command. No response from model.")?;

    let content = choice.message.content.unwrap_or_default();

    // find <command>...</command> in content
    // if cannot find assume no command

    let start_index = content.find("<command>").map(|i| i + "<command>".len());
    let end_index = content.find("</command>");

    if let (Some(start), Some(end)) = (start_index, end_index) {
        Ok(content[start..end].trim().to_string())
    } else {
        anyhow::bail!("No command could be generated for the action.")
    }
}

async fn get_openai_client() -> Result<OpenAIClient> {
    let api_key = get_api_key().await?;
    let config = OpenAIConfig::new().with_api_key(api_key);
    Ok(OpenAIClient::with_config(config))
}

const DATA_DIR_ENV_VAR: &str = "HOWTO_CLI_DATA_DIR";
const OPENAI_API_KEY_ENV_VAR: &str = "HOWTO_CLI_OPENAI_API_KEY";
const DEFAULT_DATA_DIR_NAME: &str = ".howto-cli";
const OPENAI_API_KEY_FILE: &str = "credentials";

async fn get_api_key() -> Result<String> {
    match env::var(OPENAI_API_KEY_ENV_VAR) {
        Ok(api_key) => Ok(api_key),
        Err(VarError::NotPresent) => {
            let data_dir = get_data_dir()?;
            let api_key_path = data_dir.join(OPENAI_API_KEY_FILE);
            tokio::fs::read_to_string(api_key_path)
                .await
                .context("Unable to read OpenAI API key from file")
                .map(|key| key.trim().to_string())
        }
        Err(VarError::NotUnicode(_)) => Err(anyhow::anyhow!(
            "The value of the {} environment variable is not valid Unicode.",
            OPENAI_API_KEY_ENV_VAR
        )),
    }
}

fn get_data_dir() -> Result<PathBuf> {
    match env::var(DATA_DIR_ENV_VAR) {
        Ok(data_dir) => Ok(PathBuf::from(data_dir)),
        Err(VarError::NotPresent) => {
            let home_dir = dirs::home_dir().context(format!(
                "Unable to determine home directory. Set the {} environment variable to override.",
                DATA_DIR_ENV_VAR
            ))?;
            Ok(home_dir.join(DEFAULT_DATA_DIR_NAME))
        }
        Err(VarError::NotUnicode(_)) => Err(anyhow::anyhow!(
            "The value of the {} environment variable is not valid Unicode.",
            DATA_DIR_ENV_VAR
        )),
    }
}
