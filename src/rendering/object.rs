pub trait Object {
    fn update<T>(&mut self, vertices: VertexBuffer<T>);
    fn draw(&self);
}