use peaq_gen::api::peaq_did::events::AttributeRead;
use serde::{Deserialize, Serialize};
use subxt::{
    events::{EventDetails, StaticEvent},
    PolkadotConfig,
};

#[derive(Serialize, Deserialize, Debug)]
#[allow(clippy::upper_case_acronyms)]
struct DID {
    field_1: u8,
    field_2: String,
    field_3: u64,
    field_4: bool,
}

#[tokio::main]
async fn main() {
    let client = peaq_client::Client::new("").await.unwrap();

    // Save DID.
    {
        eprintln!("starting to add attribute on-chain");
        let did = DID {
            field_1: 69,
            field_2: String::from("finally"),
            field_3: 96,
            field_4: false,
        };
        let value = serde_json::to_vec(&did).unwrap();
        client.add_attribute("staex-ioa", value).await.unwrap();
    }
    // Get DID.
    {
        eprintln!("starting to read attribute on-chain");
        let did: Option<DID> =
            client.read_attribute::<DID, _>("staex-ioa", Some(filter)).await.unwrap();
        eprintln!("{:?}", did)
    }
}

fn filter(event: EventDetails<PolkadotConfig>) -> Option<DID> {
    if event.variant_name() == AttributeRead::EVENT {
        if let Ok(Some(evt)) = event.as_event::<AttributeRead>() {
            match serde_json::from_slice(&evt.0.value) {
                Ok(did) => return Some(did),
                Err(_) => return None,
            }
        }
    }
    None
}
