use http::HeaderMap;
use http::Request;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use uclient::ClientExt;

#[cfg(any(feature = "async_surf_rustls", feature = "async_surf"))]
#[async_std::test]
async fn get() {
    use uclient::surf::SurfClient as Client;
    let mut h = HeaderMap::new();
    h.insert("Custom", "Unknown".parse().unwrap());
    let client = Client::new(h).unwrap();
    let res = client
        .get(url::Url::parse("https://httpbin.org/get").unwrap(), "")
        .await;
    assert_eq!(res.is_ok(), true, "{:?}", res);

    let result = res.unwrap();
    let json: Result<Value, _> = serde_json::from_str(result.body());
    assert_eq!(json.is_ok(), true, "{:?}", json);

    let json = json.unwrap();
    assert_eq!(json.is_object(), true, "{:?}", json);

    let json = json.as_object().unwrap();
    let headers = json.get("headers");
    assert_eq!(headers.is_some(), true, "{:?}", headers);

    let h = headers.unwrap();
    let custom_header = h.get("Custom");
    assert_eq!(custom_header.is_some(), true, "{:?}", h);

    let content = custom_header.unwrap().as_str();
    assert_eq!(content.is_some(), true, "{:?}", h);

    assert_eq!(content.unwrap(), "Unknown", "{:?}", h);
}

#[cfg(any(feature = "async_surf_rustls", feature = "async_surf"))]
#[async_std::test]
async fn post() {
    use uclient::surf::SurfClient as Client;
    let h = HeaderMap::new();
    // h.insert(http::header::CONTENT_TYPE, http);
    let client = Client::new(h).unwrap();
    let res = client
        .post(
            url::Url::parse("https://httpbin.org/post").unwrap(),
            "{\"my\":123}",
        )
        .await;
    assert_eq!(res.is_ok(), true, "{:?}", res);

    let result = res.unwrap();
    let json: Result<Value, _> = serde_json::from_str(result.body());
    assert_eq!(json.is_ok(), true, "{:?}", json);

    let json = json.unwrap();
    assert_eq!(json.is_object(), true, "{:?}", json);
    let json = json.as_object().unwrap();

    let data = json.get("data");
    assert_eq!(data.is_some(), true, "{:?}", data);

    let d = data.unwrap();
    assert_eq!(d.as_str().unwrap(), "{\"my\":123}", "{:?}", d);
}

#[cfg(all(
    any(feature = "async_surf_rustls", feature = "async_surf"),
    feature = "multipart"
))]
#[async_std::test]
async fn multipart() {
    use mime_multipart::generate_boundary;
    use uclient::form::{multipart_to_read, FilePart, FormData};
    use uclient::surf::SurfClient as Client;

    // Create a simple short file for testing
    let tmpdir = tempdir::TempDir::new("formdata_test").unwrap();
    let tmppath = tmpdir.path().join("testfile");
    let mut tmpfile = File::create(tmppath.clone()).unwrap();
    writeln!(tmpfile, "this is example file content").unwrap();

    let mut photo_headers = http::header::HeaderMap::new();
    photo_headers.insert(http::header::CONTENT_TYPE, "image/gif".parse().unwrap());

    let form = FormData {
        fields: vec![
            ("name".to_owned(), "Mike".to_owned()),
            ("age".to_owned(), "46".to_owned()),
        ],
        files: vec![("photo".to_owned(), FilePart::new(photo_headers, &tmppath))],
    };

    let stream = form.into_form_stream().unwrap();
    let boundary = stream.boundary;
    let reader = stream.reader;
    let count = stream.count;

    let client = Client::new(HeaderMap::new()).unwrap();
    let req = Request::post("https://httpbin.org/post")
        .header(http::header::CONTENT_LENGTH, count)
        .header(
            http::header::CONTENT_TYPE,
            vec![b"multipart/form-data; boundary=".to_vec(), boundary.clone()].concat(),
        )
        .body(reader)
        .unwrap();
    let res = client.request_reader(req).await;
    assert_eq!(res.is_ok(), true, "{:?}", res);

    let result = res.unwrap();
    println!("{}", result.body());

    let json: Result<Value, _> = serde_json::from_str(result.body());
    assert_eq!(json.is_ok(), true, "{:?}", json);

    let json = json.unwrap();

    let form = json
        .as_object()
        .unwrap()
        .get("form")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(
        form.get("age").unwrap().as_str().unwrap(),
        "46",
        "{:?}",
        json
    );
    assert_eq!(
        form.get("name").unwrap().as_str().unwrap(),
        "Mike",
        "{:?}",
        json
    );

    let files = json
        .as_object()
        .unwrap()
        .get("files")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(
        files.get("photo").unwrap().as_str().unwrap(),
        "this is example file content\n",
        "{:?}",
        json
    );

    println!("{}", result.body());
}
