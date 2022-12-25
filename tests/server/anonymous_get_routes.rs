use crate::helper::create_server;
use actix_web::{
    dev::{Service, ServiceResponse},
    http::{header, Method, StatusCode},
    test,
};

#[actix_web::test]
async fn test_index_get() {
    let app = test::init_service(create_server().await).await;

    // Create request object
    let req = test::TestRequest::with_uri("/")
        .method(Method::GET)
        .to_request();

    // Execute application
    let res = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_logout_get() {
    let app = test::init_service(create_server().await).await;

    // Create request object
    let req = test::TestRequest::with_uri("/logout")
        .method(Method::GET)
        .to_request();

    // Execute application
    let res = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_login_get() {
    let app = test::init_service(create_server().await).await;

    // Create request object
    let req = test::TestRequest::with_uri("/login")
        .method(Method::GET)
        .to_request();

    // Execute application
    let res = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_register_get() {
    let app = test::init_service(create_server().await).await;

    // Create request object
    let req = test::TestRequest::with_uri("/register")
        .method(Method::GET)
        .to_request();

    // Execute application
    let res = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_settings_get() {
    let app = test::init_service(create_server().await).await;

    // Create request object
    let req = test::TestRequest::with_uri("/settings")
        .method(Method::GET)
        .to_request();

    // Execute application
    let res: ServiceResponse = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FOUND);
    assert_eq!(
        res.headers().get(header::LOCATION),
        Some(&header::HeaderValue::from_static("/login"))
    );
}

#[actix_web::test]
async fn test_editor_get() {
    let app = test::init_service(create_server().await).await;

    // Create request object
    let req = test::TestRequest::with_uri("/editor")
        .method(Method::GET)
        .to_request();

    // Execute application
    let res = app.call(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FOUND);
    assert_eq!(
        res.headers().get(header::LOCATION),
        Some(&header::HeaderValue::from_static("/login"))
    );
}
