use std::collections::HashMap;

/// This function prompts the user for their FAS password.
pub fn read_password() -> String {
    rpassword::prompt_password_stdout("FAS Password: ").expect("Failed to read password from stdin.")
}

/// This function asks for and stores the password in the session keyring.
pub fn get_store_password(clear: bool) -> Result<String, String> {
    let ss = match secret_service::SecretService::new(secret_service::EncryptionType::Dh) {
        Ok(ss) => ss,
        Err(error) => {
            println!("Failed to initialize SecretService client: {}", error);
            return Ok(read_password());
        },
    };

    let collection = match ss.get_default_collection() {
        Ok(c) => c,
        Err(error) => {
            println!("Failed to query SecretService: {}", error);
            return Ok(read_password());
        },
    };

    let mut attributes = HashMap::new();
    attributes.insert("fedora-update-feedback", "FAS Password");

    let store = |password: &str, replace: bool| {
        if let Err(error) = collection.create_item(
            "fedora-update-feedback",
            attributes.clone(),
            password.as_bytes(),
            replace,
            "password",
        ) {
            println!("Failed to save password with SecretService: {}", error);
        }
    };

    let items = match collection.search_items(attributes.clone()) {
        Ok(items) => items,
        Err(error) => {
            format!("Failed to query SecretService: {}", error);
            return Ok(read_password());
        },
    };

    if clear {
        let password = read_password();
        store(&password, true);
        return Ok(password);
    };

    let password = match items.get(0) {
        Some(item) => match item.get_secret() {
            Ok(secret) => match String::from_utf8(secret) {
                Ok(valid) => valid,
                Err(error) => {
                    println!("Stored password was not valid UTF-8: {}", error);

                    let password = read_password();
                    store(&password, true);

                    password
                },
            },
            Err(error) => {
                println!("Password was not stored correctly: {}", error);

                let password = read_password();
                store(&password, true);

                password
            },
        },
        None => {
            let password = read_password();
            store(&password, false);

            password
        },
    };

    Ok(password)
}
