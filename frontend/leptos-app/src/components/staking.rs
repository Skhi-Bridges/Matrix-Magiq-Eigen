use leptos::*;
use permaweb_lib::profile::{Profile, Zone, Wallet};

// Staking Component
#[component]
pub fn StakingInterface() -> impl IntoView {
    // ActorX implementation with Permaweb Profile
    let profile = Profile::new("Staking-Component");
    let zone = Zone::new(&profile);
    let wallet = Wallet::new(&profile);
    
    view! {
        <div class="staking-interface">
            <h2>"EigenLayer Staking"</h2>
            <div class="staking-controls">
                // Staking interface components
            </div>
        </div>
    }
}

// Validator Component
#[component]
pub fn ValidatorInterface() -> impl IntoView {
    // ActorX implementation with Permaweb Profile
    let profile = Profile::new("Validator-Component");
    let zone = Zone::new(&profile);
    let wallet = Wallet::new(&profile);
    
    view! {
        <div class="validator-interface">
            <h2>"EigenLayer Validators"</h2>
            <div class="validator-controls">
                // Validator interface components
            </div>
        </div>
    }
}
