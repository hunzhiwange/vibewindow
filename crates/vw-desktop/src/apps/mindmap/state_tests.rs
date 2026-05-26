use super::state::{
    BracketLayoutFormat, FishboneLayoutFormat, MindMapDiagramType, MindMapLayoutFormat,
    OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};

#[test]
fn layout_label_methods_cover_each_variant() {
    assert_eq!(MindMapLayoutFormat::RightAligned.label(), "右侧展开");
    assert_eq!(MindMapLayoutFormat::LeftAligned.label(), "左侧展开");
    assert_eq!(MindMapLayoutFormat::Bidirectional.label(), "双侧展开");
    assert_eq!(OrgChartLayoutFormat::TopDown.label(), "自上而下（曲线）");
    assert_eq!(OrgChartLayoutFormat::LeftRight.label(), "自上而下（折线）");
    assert_eq!(FishboneLayoutFormat::HeadRight.label(), "鱼头在右");
    assert_eq!(FishboneLayoutFormat::HeadLeft.label(), "鱼头在左");
    assert_eq!(BracketLayoutFormat::BraceRight.label(), "括号在右");
    assert_eq!(BracketLayoutFormat::BraceLeft.label(), "括号在左");
    assert_eq!(TimelineLayoutFormat::UpDown.label(), "上下");
    assert_eq!(TimelineLayoutFormat::AllUp.label(), "全上");
    assert_eq!(TimelineLayoutFormat::AllDown.label(), "全下");
    assert_eq!(TreeLayoutFormat::SymmetricSplit.label(), "左右对称分支");
    assert_eq!(TreeLayoutFormat::FanDown.label(), "顶部中心多分支");
    assert_eq!(TreeLayoutFormat::LeftAligned.label(), "左侧单边分支");
    assert_eq!(TreeLayoutFormat::RightAligned.label(), "右侧单边分支");
}

#[test]
fn diagram_type_label_methods_cover_each_variant() {
    assert_eq!(MindMapDiagramType::MindMap.label(), "思维导图");
    assert_eq!(MindMapDiagramType::OrgChart.label(), "组织结构图");
    assert_eq!(MindMapDiagramType::Fishbone.label(), "鱼骨图");
    assert_eq!(MindMapDiagramType::Timeline.label(), "时间轴");
    assert_eq!(MindMapDiagramType::Tree.label(), "树形图");
    assert_eq!(MindMapDiagramType::Bracket.label(), "括号图");
}
