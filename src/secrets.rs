use std::collections::HashMap;

use secret_service::{Collection, EncryptionType, SecretService};

/// This function prompts the user for their FAS password.
pub fn read_password() -> String {
    rpassword::prompt_password("FAS Password: ").expect("Failed to read password from stdin.")
}

/// This function stores the password in the session keyring.
async fn store_password(
    collection: &mut Collection<'_>,
    attributes: HashMap<&str, &str>,
    password: &[u8],
    replace: bool,
) {
    if let Err(error) = collection
        .create_item("bodhi-cli", attributes.clone(), password, replace, "password")
        .await
    {
        println!("Failed to save password with SecretService: {}", error);
    }
}

/// This function asks for and stores the password in the session keyring.
pub(crate) async fn get_store_password(clear: bool) -> Result<String, String> {
    let ss = match SecretService::connect(EncryptionType::Dh).await {
        Ok(ss) => ss,
        Err(error) => {
            println!("Failed to initialize SecretService client: {}", error);
            return Ok(read_password());
        },
    };

    let mut collection = match ss.get_default_collection().await {
        Ok(c) => c,
        Err(error) => {
            println!("Failed to query SecretService: {}", error);
            return Ok(read_password());
        },
    };

    let mut attributes = HashMap::new();
    attributes.insert("bodhi-cli", "FAS Password");

    let items = match collection.search_items(attributes.clone()).await {
        Ok(items) => items,
        Err(error) => {
            format!("Failed to query SecretService: {}", error);
            return Ok(read_password());
        },
    };

    if clear {
        let password = read_password();
        store_password(&mut collection, attributes, password.as_bytes(), true).await;
        return Ok(password);
    };

    let password = match items.first() {
        Some(item) => match item.get_secret().await {
            Ok(secret) => match String::from_utf8(secret) {
                Ok(valid) => valid,
                Err(error) => {
                    println!("Stored password was not valid UTF-8: {}", error);
                    let password = read_password();
                    store_password(&mut collection, attributes, password.as_bytes(), true).await;
                    password
                },
            },
            Err(error) => {
                println!("Password was not stored correctly: {}", error);
                let password = read_password();
                store_password(&mut collection, attributes, password.as_bytes(), true).await;
                password
            },
        },
        None => {
            let password = read_password();
            store_password(&mut collection, attributes, password.as_bytes(), false).await;

            password
        },
    };

    Ok(password)
}
