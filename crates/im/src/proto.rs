pub mod request {
    include!(concat!(env!("OUT_DIR"), "/request.rs"));
}

pub mod response {
    include!(concat!(env!("OUT_DIR"), "/response.rs"));
}
