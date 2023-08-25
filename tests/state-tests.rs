#[cfg(feature = "state")]
use std::collections::HashMap;
#[cfg(feature = "state")]
use std::sync::Arc;
#[cfg(feature = "state")]
use std::thread;
#[cfg(feature = "state")]
use std::time::Duration;

#[cfg(feature = "state")]
use pipewire_wrapper::core_api::client::ClientRef;
#[cfg(feature = "state")]
use pipewire_wrapper::core_api::core::Core;
#[cfg(feature = "state")]
use pipewire_wrapper::core_api::device::DeviceRef;
#[cfg(feature = "state")]
use pipewire_wrapper::core_api::link::LinkRef;
#[cfg(feature = "state")]
use pipewire_wrapper::core_api::node::NodeRef;
#[cfg(feature = "state")]
use pipewire_wrapper::core_api::port::PortRef;
#[cfg(feature = "state")]
use pipewire_wrapper::core_api::proxy::Proxied;
#[cfg(feature = "state")]
use pipewire_wrapper::core_api::type_info::TypeInfo;
#[cfg(feature = "state")]
use pipewire_wrapper::spa::param::ParamType;
#[cfg(feature = "state")]
use pipewire_wrapper::state::State;

#[test]
#[cfg(feature = "state")]
fn test() {
    let core = Core::default();
    let context = core.context();
    let main_loop = context.main_loop();
    let registry = core.get_registry(0).unwrap();

    let _timer = main_loop.quit_after(Duration::from_secs(1)).unwrap();

    let subscriptions = vec![
        NodeRef::type_info(),
        PortRef::type_info(),
        LinkRef::type_info(),
        DeviceRef::type_info(),
        ClientRef::type_info(),
    ];
    let mut params_subscriptions: HashMap<TypeInfo<'static>, Vec<ParamType>> = HashMap::default();
    params_subscriptions.insert(NodeRef::type_info(), ParamType::all().to_vec());
    params_subscriptions.insert(PortRef::type_info(), ParamType::all().to_vec());
    params_subscriptions.insert(DeviceRef::type_info(), ParamType::all().to_vec());
    let mut state = State::new(
        core.clone(),
        registry.clone(),
        subscriptions,
        params_subscriptions,
    );
    let messages_receiver = state.create_channel();
    state.subscribe_changes();
    let state = Arc::new(state);
    thread::spawn({
        let _state = state.clone();
        move || loop {
            messages_receiver
                .iter()
                .for_each(|r| println!("State update: {:?}", r))
        }
    });

    main_loop.run().unwrap();
}
