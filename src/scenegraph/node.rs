pub enum NodeType {
    Mesh,
    Texture,
    Material,
    MaterialParameters,
    Camera,
    Light,
}

trait Node {
    fn id(&self);
    fn parent(&self) -> Node;
    fn children(&self) -> Vec<Node>;
    fn node_type(&self) -> NodeType;
}
