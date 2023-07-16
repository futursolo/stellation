use std::time::Duration;

use time::macros::format_description;
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::prelude::*;

use crate::api::{Bridge, ServerTimeQuery};

#[function_component]
pub fn ServerTime() -> HtmlResult {
    let server_time = Bridge::use_query::<ServerTimeQuery>(().into())?;
    {
        let server_time = server_time.clone();

        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    loop {
                        sleep(Duration::from_secs(1)).await;
                        let _ = server_time.refresh().await;
                    }
                });
            },
            (),
        );
    }

    let server_time = match server_time.as_deref() {
        Ok(m) => m
            .value
            .format(format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ))
            .expect("failed to format time!"),
        Err(_) => {
            return Ok(html! {
                <div>{"Waiting for Server..."}</div>
            })
        }
    };

    Ok(html! {
        <div>{"Server Time: "}{server_time}</div>
    })
}
