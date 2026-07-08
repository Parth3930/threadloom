use threadloom_core::{create_signal, IntoView, View};
use threadloom_macro::threadloom;
use threadloom_ui::*;

pub fn page() -> View {
    let (error_msg, set_error_msg) = create_signal(String::new());
    let (rooms, set_rooms) = create_signal(Vec::<crate::api::list_rooms::route::RoomInfo>::new());
    let (_rooms_loaded, set_rooms_loaded) = create_signal(false);
    let (confirm_room, set_confirm_room) =
        create_signal(Option::<crate::api::list_rooms::route::RoomInfo>::None);

    {
        let set_rooms = set_rooms;
        threadloom_dom::wasm_bindgen_futures::spawn_local(async move {
            // Try the cookie cache first — no network needed.
            if let Some(cached) = crate::store::load_cached_rooms() {
                set_rooms.set(cached);
                set_rooms_loaded.set(true);
                let _ = threadloom_dom::tick();
                return; // cache hit: skip the backend entirely
            }

            // No cache: fetch from backend (first visit or cache cleared).
            let tok = crate::store::AuthState::get();
            if tok.is_empty() {
                set_rooms.set(vec![]);
                set_rooms_loaded.set(true);
                let _ = threadloom_dom::tick();
                return;
            }

            match crate::api::list_rooms::route::list_rooms(
                crate::api::list_rooms::route::ListRoomsArgs { token: tok },
            )
            .await
            {
                Ok(list) => {
                    web_sys::console::log_1(
                        &format!("Fetched {} rooms from DB!", list.len()).into(),
                    );
                    // Persist to cookie cache so the next refresh is instant.
                    crate::store::save_cached_rooms(&list);
                    set_rooms.set(list);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to fetch rooms: {}", e).into());
                }
            }
            set_rooms_loaded.set(true);
            // Flush pending dynamic boundaries (the rooms list block) so the UI
            // re-renders. set_rooms.set() only marks the boundary dirty; tick()
            // is what actually re-evaluates it. Without this the console logs
            // "Fetched N rooms" but the DOM stays empty.
            let _ = threadloom_dom::tick();
        });
    }

    threadloom! {
        Section(row=false, height="screen", width="full", class="bg-background text-foreground flex flex-col items-center justify-center relative px-4") {

            // Header/Logo
            Row(items="center", justify="between", class="absolute top-6 left-6 right-6") {
                Row(items="center", gap=3) {
                    Image(src="/assets/favicon.svg", alt="Collab Whiteboard Logo", class="w-8 h-8") {}
                    Text(variant="span", class="font-bold text-xl tracking-tight hidden sm:inline-block") { "Collab Whiteboard" }
                }

                { move || {
                    let is_auth = !crate::store::AuthState::get().is_empty();
                    if is_auth {
                        threadloom! {
                            Button(label="Log Out", primary=false, class="text-sm px-4 py-1.5", on_click=move || {
                                threadloom_dom::set_cookie!("auth_token", "", 0);
                                crate::store::clear_cached_rooms();
                                crate::store::AuthState::set(String::new());
                            })
                        }
                    } else {
                        threadloom! {
                            Row(gap=3) {
                                { threadloom_core::element("a").attr("href", "/login").attr("class", "text-sm font-medium hover:underline px-3 py-1.5").child("Log In").into_view() }
                                { threadloom_core::element("a").attr("href", "/signup").attr("class", "text-sm font-medium bg-primary text-primary-foreground hover:opacity-90 px-4 py-1.5 shadow-sm rounded-none").child("Sign Up").into_view() }
                            }
                        }
                    }
                }}
            }

            // Main Card
            Section(row=false, class="w-full max-w-md border border-border rounded-xl bg-card shadow-sm p-8 flex flex-col gap-6") {
                Heading(level=2, class="text-3xl font-extrabold tracking-tight text-center") {
                    "Join a Room"
                }

                Text(variant="p", class="text-muted-foreground text-center text-sm") {
                    "Enter a Room ID to start drawing collaboratively in real-time."
                }

                Column(items="center", gap=4, class="w-full mt-4") {
                    Input(
                        id="room_id_input",
                        placeholder="e.g. daily-standup",
                        class="w-full"
                    ) {}

                    Row(gap=3, class="w-full") {
                        Button(
                            label="Join Room",
                            primary=true,
                            class="flex-1 py-2 flex items-center justify-center text-sm font-semibold",
                            on_click=move || {
                                let val = threadloom_dom::get_value!("room_id_input");

                                if !val.is_empty() {
                                    threadloom_dom::wasm_bindgen_futures::spawn_local(async move {
                                        match crate::api::join_room::route::join_room(crate::api::join_room::route::JoinRoomArgs { id: val.clone() }).await {
                                            Ok(_) => threadloom::navigate!(&format!("/board?room={}", val)),
                                            Err(e) => set_error_msg.set(e),
                                        }
                                        let _ = threadloom_dom::tick();
                                    });
                                } else {
                                    set_error_msg.set("Please enter a room ID to join".to_string());
                                }
                            }
                        )
                        Button(
                            label="Create Room",
                            primary=false,
                            class="flex-1 py-2 flex items-center justify-center text-sm font-semibold bg-secondary text-secondary-foreground hover:bg-secondary/80",
                            on_click=move || {
                                let val = threadloom_dom::get_value!("room_id_input");
                                let tok = crate::store::AuthState::get();

                                if tok.is_empty() {
                                    set_error_msg.set("You must log in to create a room".to_string());
                                    return;
                                }

                                let set_rooms = set_rooms;
                                threadloom_dom::wasm_bindgen_futures::spawn_local(async move {
                                    match crate::api::create_room::route::create_room(crate::api::create_room::route::CreateRoomArgs { name: val, token: tok.clone() }).await {
                                        Ok(room_id) => {
                                            // Refresh rooms list
                                            if let Ok(list) = crate::api::list_rooms::route::list_rooms(
                                                crate::api::list_rooms::route::ListRoomsArgs { token: tok }
                                            ).await {
                                                crate::store::save_cached_rooms(&list);
                                                set_rooms.set(list);
                                            }
                                            threadloom::navigate!(&format!("/board?room={}", room_id));
                                        }
                                        Err(e) => set_error_msg.set(e),
                                    }
                                    let _ = threadloom_dom::tick();
                                });
                            }
                        )
                    }
                }

                // Error Message
                { move || {
                    let msg = error_msg.get();
                    if msg.is_empty() {
                        threadloom_core::element("div").into_view()
                    } else {
                        threadloom_core::element("p").attr("class", "text-destructive text-sm text-center font-medium mt-2").child(msg).into_view()
                    }
                }}
            }

            // Your Rooms List
            { move || {
                let room_list = rooms.get();

                let mut section = threadloom_core::element("div")
                    .attr("class", "w-full max-w-md mt-6 flex flex-col gap-2 min-h-[100px]");

                if !room_list.is_empty() {
                    let heading = threadloom_core::element("p")
                        .attr("class", "text-xs font-semibold text-muted-foreground uppercase tracking-widest mb-1")
                        .child("Your Rooms");
                    section = section.child(heading);

                    for room in room_list.iter() {
                        let room_id = room.id.clone();
                        let room_name = room.name.clone();
                        let href = format!("/board?room={}", room_id);

                        // Row container (not an anchor, so the delete button
                        // doesn't trigger navigation).
                        let mut row = threadloom_core::element("div")
                            .attr("class", "flex items-center justify-between px-4 py-3 rounded-lg border border-border bg-card hover:bg-muted/50 transition-colors");

                        // Main link: name only
                        let link = threadloom_core::element("a")
                            .attr("href", href)
                            .attr("class", "flex items-center justify-between flex-1 min-w-0")
                            .child(
                                threadloom_core::element("span")
                                    .attr("class", "text-sm font-medium truncate")
                                    .child(room_name.clone())
                            );
                        row = row.child(link);

                        // Delete button — opens a confirmation modal instead of
                        // deleting immediately.
                        let del_room = room.clone();
                        let set_confirm_room = set_confirm_room;
                        let del_btn = threadloom_core::element("button")
                            .attr("type", "button")
                            .attr("title", "Delete room")
                            .attr("class", "ml-3 shrink-0 text-muted-foreground hover:text-destructive transition-colors")
                            .child("🗑️")
                            .on("click", move || {
                                set_confirm_room.set(Some(del_room.clone()));
                                let _ = threadloom_dom::tick();
                            });
                        row = row.child(del_btn);

                        section = section.child(row);
                    }
                }

                section.into_view()
            }}

            { move || {
                let target = confirm_room.get();
                match target {
                    None => threadloom_core::element("div").into_view(),
                    Some(room) => {
                        let room_name = room.name.clone();
                        let room_id = room.id.clone();

                        // Cancel
                        let set_confirm_room_cancel = set_confirm_room;
                        let cancel_btn = threadloom_core::element("button")
                            .attr("type", "button")
                            .attr("class", "px-4 py-2 text-sm rounded-md border border-border hover:bg-muted transition-colors")
                            .child("Cancel")
                            .on("click", move || {
                                set_confirm_room_cancel.set(None);
                                let _ = threadloom_dom::tick();
                            });

                        // Confirm delete
                        let set_confirm_room_ok = set_confirm_room;
                        let set_rooms_ok = set_rooms;
                        let delete_btn = threadloom_core::element("button")
                            .attr("type", "button")
                            .attr("class", "px-4 py-2 text-sm rounded-md bg-destructive text-destructive-foreground hover:opacity-90 transition-opacity")
                            .child("Delete")
                            .on("click", move || {
                                let id = room_id.clone();
                                let tok = crate::store::AuthState::get();
                                let set_rooms_ok = set_rooms_ok;
                                let set_confirm_room_ok = set_confirm_room_ok;
                                threadloom_dom::wasm_bindgen_futures::spawn_local(async move {
                                    match crate::api::delete_room::route::delete_room(
                                        crate::api::delete_room::route::DeleteRoomArgs { id: id.clone(), token: tok }
                                    ).await {
                                        Ok(_) => {
                                            if let Ok(list) = crate::api::list_rooms::route::list_rooms(
                                                crate::api::list_rooms::route::ListRoomsArgs { token: crate::store::AuthState::get() }
                                            ).await {
                                                crate::store::save_cached_rooms(&list);
                                                set_rooms_ok.set(list);
                                            }
                                        }
                                        Err(e) => web_sys::console::error_1(&format!("Failed to delete room: {}", e).into()),
                                    }
                                    set_confirm_room_ok.set(None);
                                    let _ = threadloom_dom::tick();
                                });
                            });

                        threadloom_core::element("div")
                            .attr("class", "fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-4")
                            .child(
                                threadloom_core::element("div")
                                    .attr("class", "w-full max-w-sm rounded-xl border border-border bg-card shadow-lg p-6 flex flex-col gap-4")
                                    .child(
                                        threadloom_core::element("h3")
                                            .attr("class", "text-lg font-semibold")
                                            .child("Delete room?")
                                    )
                                    .child(
                                        threadloom_core::element("p")
                                            .attr("class", "text-sm text-muted-foreground")
                                            .child(format!("This will permanently delete \" {}\" and all of its drawings. This cannot be undone.", room_name))
                                    )
                                    .child(
                                        threadloom_core::element("div")
                                            .attr("class", "flex justify-end gap-3 mt-2")
                                            .child(cancel_btn)
                                            .child(delete_btn)
                                    )
                            )
                            .into_view()
                    }
                }
            }}

            Text(variant="p", class="text-xs text-muted-foreground mt-8 font-mono absolute bottom-6") {
                "Powered by Threadloom & Turso"
            }
        }
    }
}
