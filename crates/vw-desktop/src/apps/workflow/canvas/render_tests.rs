use super::*;

#[test]
fn render_helpers_are_wired_through_canvas_module() {
    let _grid: fn(&mut Frame, Size, Vector, f32, &Theme) = draw_grid;
    let _edges: fn(
        &mut Frame,
        &WorkflowDocument,
        Vector,
        f32,
        &Theme,
        Option<&str>,
        Option<&str>,
        Option<&str>,
        &HandleSlots,
    ) = draw_edges;
    let _draft: fn(
        &mut Frame,
        &WorkflowDocument,
        Vector,
        f32,
        Option<&WorkflowConnectionDraft>,
        &HandleSlots,
    ) = draw_connection_draft;
    let _nodes: fn(
        &mut Frame,
        &WorkflowDocument,
        Vector,
        f32,
        Option<&str>,
        Option<&str>,
        Option<&str>,
        Option<&WorkflowConnectionEndpoint>,
        &Theme,
        Color,
        &HandleSlots,
    ) = draw_nodes;
}
