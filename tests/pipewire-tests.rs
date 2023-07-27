/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::CString;

use pipewire_wrapper::core_api::PipeWire;

#[test]
fn test_init() {
    let arg = CString::new("test_arg").unwrap();
    let pw = PipeWire::init(&vec![&arg]);

    println!("Spa support{:?}", pw.get_spa_support(5));
    println!("User name {:?}", pw.get_user_name());
    println!("Program name {:?}", pw.get_prgname());
    println!("Application name {:?}", pw.get_application_name());
    println!("Client name {:?}", pw.get_client_name());
    println!("Host name {:?}", pw.get_host_name());
    println!("Domain name {:?}", pw.get_domain());
    println!(
        "Set domain {:?}",
        pw.set_domain(CString::new("test_domain").unwrap().as_ref())
    );
    assert_eq!(
        pw.get_domain().unwrap(),
        CString::new("test_domain").unwrap().as_ref()
    );
    assert!(!pw.debug_is_category_enabled(&CString::new("wrong_debug_category").unwrap()));
    println!("In valgrind {:?}", pw.in_valgrind());
}
