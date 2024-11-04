use torrent_parser::parse_torrent_file;

#[test]
fn test_parse_metadata() {
    let file = "../test.torrent";
    let metadata = parse_torrent_file(file).unwrap();
}

#[test]
fn test_info_hash() {
    let file = "../test.torrent";
    let info_hash = parse_torrent_file(file).unwrap().info_hash;

    dbg!(info_hash);
}
