use std::io::Read;
use std::path::PathBuf;

use smithay_client_toolkit::data_device_manager::{
    data_device::DataDeviceHandler,
    data_offer::{DataOfferHandler, DragOffer},
    data_source::DataSourceHandler,
    WritePipe,
};
use wayland_client::{
    Connection, QueueHandle,
    protocol::{
        wl_data_device::WlDataDevice,
        wl_data_device_manager::DndAction,
        wl_data_source::WlDataSource,
        wl_surface::WlSurface,
    },
};

use super::WaylandState;

// file managers offer dragged files as a list of file:// links
const URI_LIST: &str = "text/uri-list";

impl DataDeviceHandler for WaylandState {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataDevice,
        _: f64,
        _: f64,
        _: &WlSurface,
    ) {
        let Some(offer) = self.drag_offer() else {
            return;
        };

        let has_files = offer.with_mime_types(|types| types.iter().any(|kind| kind == URI_LIST));

        // anything but files is turned down, so the pointer shows it can't be dropped here
        if !has_files {
            offer.accept_mime_type(offer.serial, None);

            return;
        }

        offer.accept_mime_type(offer.serial, Some(String::from(URI_LIST)));
        offer.set_actions(DndAction::Copy, DndAction::Copy);
    }

    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}

    fn motion(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice, _: f64, _: f64) {}

    fn selection(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}

    fn drop_performed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {
        let Some(offer) = self.drag_offer() else {
            return;
        };

        let Ok(mut pipe) = offer.receive(String::from(URI_LIST)) else {
            offer.destroy();

            return;
        };

        // the receive request has to reach the other program before it can write anything
        let _ = self.connection.flush();

        /*
         * ponytail: blocks the event loop until the other program has written
         * the list, which is instant for file managers; read it on the loop if
         * a slow source ever stalls a frame
         */
        let mut list = String::new();

        let read = pipe.read_to_string(&mut list);

        offer.finish();
        offer.destroy();

        if read.is_err() {
            return;
        }

        let files = parse(&list);

        if files.is_empty() {
            return;
        }

        let surface = offer.surface.clone();

        let Some(window) = self.window(&surface) else {
            return;
        };

        let x = offer.x as f32;
        let y = offer.y as f32;

        if window.pointer.drop_files(x, y, files) {
            self.request_frame_on(&surface);
        }
    }
}

// only drags into amane are handled, so the offer's choices need no answer
impl DataOfferHandler for WaylandState {
    fn source_actions(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        _: DndAction,
    ) {
    }

    fn selected_action(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        _: DndAction,
    ) {
    }
}

// amane never offers data of its own
impl DataSourceHandler for WaylandState {
    fn accept_mime(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataSource,
        _: Option<String>,
    ) {
    }

    fn send_request(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataSource,
        _: String,
        _: WritePipe,
    ) {
    }

    fn cancelled(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {}

    fn dnd_dropped(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {}

    fn dnd_finished(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {}

    fn action(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource, _: DndAction) {}
}

impl WaylandState {
    fn drag_offer(&self) -> Option<DragOffer> {
        self.data_device.as_ref()?.data().drag_offer()
    }
}

// one link per line, lines starting with # are comments
fn parse(list: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for line in list.lines() {
        let line = line.trim();

        let Some(path) = line.strip_prefix("file://") else {
            continue;
        };

        // a host may come before the path, like file://laptop/home
        let Some(start) = path.find('/') else {
            continue;
        };

        files.push(PathBuf::from(decode(&path[start..])));
    }

    files
}

// spaces and other special letters arrive as %20 and the like
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();

    let mut decoded = Vec::new();

    let mut index = 0;

    while index < bytes.len() {
        let escaped = bytes[index] == b'%' && index + 2 < bytes.len();

        let value = if escaped {
            std::str::from_utf8(&bytes[index + 1..index + 3])
                .ok()
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        } else {
            None
        };

        let Some(value) = value else {
            decoded.push(bytes[index]);
            index += 1;

            continue;
        };

        decoded.push(value);
        index += 3;
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_uri_list() {
        let list = "# from a file manager\r\nfile:///home/me/My%20Clip.mov\r\nfile://host/tmp/a%2\r\nhttps://x\r\n";

        let files = parse(list);

        assert_eq!(files, [PathBuf::from("/home/me/My Clip.mov"), PathBuf::from("/tmp/a%2")]);
    }
}
