use std::time::Duration;

use example_fullstack_api::ServerTimeQuery;
use stackable_bridge::hooks::use_bridged_query;
use time::macros::format_description;
use yew::platform::spawn_local;
use yew::platform::time::sleep;
use yew::prelude::*;

#[function_component]
pub fn ServerTime() -> HtmlResult {
    let server_time = use_bridged_query::<ServerTimeQuery>(().into())?;
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
        Err(_) => panic!("failed to fetch server value!"),
    };

    Ok(html! {
        <div class="time-content">{"Server Time: "}{server_time}</div>
    })
}
