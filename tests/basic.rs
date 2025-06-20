use tao_db::{service::TaoService, db::Database, viewer::ViewerContext, tao::{NodeData}};

#[test]
fn test_insert_and_query() {
    let db = Database::new(":memory:", 4).unwrap();
    let mut service = TaoService::new(db);
    service.init().unwrap();

    let a_data = NodeData::new(Some("A".into()));
    let b_data = NodeData::new(Some("B".into()));
    let a = service.create_node_thrift("user", Some(&a_data)).unwrap();
    let b = service.create_node_thrift("user", Some(&b_data)).unwrap();
    service.create_edge_thrift(a.id, b.id, "friend", None).unwrap();

    let viewer = ViewerContext::new(a.id);
    let edges = service.get_edges(&viewer, a.id).unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].target, b.id);
}

