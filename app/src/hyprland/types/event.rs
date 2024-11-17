#[derive(Debug, PartialEq)]
pub enum HyprEvent {
    WorkspaceCreated,
    WorkspaceChanged,
    WorkspaceDestroyed,
    WindowChanged,
    Window2Changed,
    IgnoredEvent,
    // More events will be handle in the future
}

pub struct HyprEventList(Vec<HyprEvent>);

impl From<&[u8]> for HyprEventList {
    fn from(buffer: &[u8]) -> Self {
        let mut evt_list: Vec<HyprEvent> = String::from_utf8_lossy(buffer)
            .lines()
            .map(|line| match line.split(">>").next().unwrap_or_default() {
                "activewindow" => HyprEvent::WindowChanged,
                "workspace" => HyprEvent::WorkspaceChanged,
                "activewindowv2" => HyprEvent::Window2Changed,
                "createworkspace" => HyprEvent::WorkspaceCreated,
                "destroyworkspace" => HyprEvent::WorkspaceDestroyed,
                _ => HyprEvent::IgnoredEvent,
            })
            .collect();
        evt_list.dedup();
        Self(evt_list)
    }
}

impl HyprEventList {
    pub fn contains(&self, event: &HyprEvent) -> bool {
        self.0.contains(event)
    }

    pub fn contains_at_least(&self, events: &[&HyprEvent]) -> bool {
        events.iter().any(|&event| self.contains(event))
    }
}
