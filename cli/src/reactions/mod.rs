mod window;
pub use window::*;

use crate::reactions::data_reaction::DataReaction;
mod data_reaction;

macro_rules! define_reactions {
    ($($x: ty), +) => {
        pub fn run_reactions(data: &[crate::climate_data::ClimateData]) -> () {
          $({
            if <$x>::validate(data) {
              tokio::spawn(async move {
                if let Err(e) = <$x>::run().await {
                  tracing::error!("Error running reaction {}: {}", std::any::type_name::<$x>(), e)
                }
              });
            }
          })+
        }

    };
}

define_reactions!(WindowData);
