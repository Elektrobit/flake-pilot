pub fn register(container: &String, app: &String, target: Option<&String>) {
    // TODO: implement symlink setup for app registration
    println!("register: {:?} {:?} {:?}", container, app, target);
}

pub fn remove(app: &str) {
    // TODO: implement removal of symlink setup for registered app
    println!("remove: {:?}", app);
}

pub fn purge(container: &str) {
    // TODO: implement removal of all app registered against
    // the given container and also purge the container from
    // the local registry
    println!("purge: {:?}", container);
}
