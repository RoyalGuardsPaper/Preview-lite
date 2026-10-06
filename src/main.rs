use adw::prelude::*;
use gtk::prelude::*;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

#[derive(Clone)]
enum Document {
    Pdf(poppler::Document),
    Image(gdk_pixbuf::Pixbuf),
}

#[derive(Clone)]
struct Viewer {
    window: adw::ApplicationWindow,
    canvas: gtk::DrawingArea,
    viewport: gtk::ScrolledWindow,
    sidebar: gtk::Box,
    sidebar_scroller: gtk::ScrolledWindow,
    page_label: gtk::Label,
    zoom_label: gtk::Label,
    document: Rc<RefCell<Option<Document>>>,
    page: Rc<RefCell<i32>>,
    zoom: Rc<RefCell<f64>>,
}

impl Viewer {
    fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Preview Lite")
            .default_width(1080)
            .default_height(760)
            .build();

        let header = adw::HeaderBar::new();
        let open = gtk::Button::with_label("Open");
        open.set_tooltip_text(Some("Open a PDF or image (Ctrl+O)"));
        header.pack_start(&open);
        let title = gtk::Label::new(Some("Preview Lite"));
        title.add_css_class("title");
        header.set_title_widget(Some(&title));

        let previous = gtk::Button::with_label("‹");
        previous.set_tooltip_text(Some("Previous page (Page Up)"));
        let page_label = gtk::Label::new(Some("— / —"));
        let next = gtk::Button::with_label("›");
        next.set_tooltip_text(Some("Next page (Page Down)"));
        let zoom_out = gtk::Button::with_label("−");
        let zoom_label = gtk::Label::new(Some("100%"));
        let zoom_in = gtk::Button::with_label("+");
        let fit = gtk::Button::with_label("Fit");
        for control in [
            &previous,
            &page_label,
            &next,
            &zoom_out,
            &zoom_label,
            &zoom_in,
            &fit,
        ] {
            header.pack_end(control);
        }

        let canvas = gtk::DrawingArea::new();
        canvas.set_content_width(640);
        canvas.set_content_height(820);
        canvas.set_hexpand(true);
        canvas.set_vexpand(true);
        let viewport = gtk::ScrolledWindow::builder()
            .child(&canvas)
            .hexpand(true)
            .vexpand(true)
            .build();
        viewport.add_css_class("view");

        let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 8);
        sidebar.set_margin_top(10);
        sidebar.set_margin_bottom(10);
        sidebar.set_margin_start(8);
        sidebar.set_margin_end(8);
        let side_scroll = gtk::ScrolledWindow::builder()
            .child(&sidebar)
            .width_request(150)
            .vexpand(true)
            .build();
        side_scroll.set_visible(false);

        let split = gtk::Paned::new(gtk::Orientation::Horizontal);
        split.set_start_child(Some(&side_scroll));
        split.set_end_child(Some(&viewport));
        split.set_resize_start_child(false);
        split.set_shrink_start_child(false);

        let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        layout.append(&header);
        layout.append(&split);
        window.set_content(Some(&layout));

        let viewer = Self {
            window,
            canvas,
            viewport,
            sidebar,
            sidebar_scroller: side_scroll,
            page_label,
            zoom_label,
            document: Rc::new(RefCell::new(None)),
            page: Rc::new(RefCell::new(0)),
            zoom: Rc::new(RefCell::new(1.0)),
        };
        viewer.connect_canvas();

        let v = viewer.clone();
        open.connect_clicked(move |_| v.choose_file());
        let v = viewer.clone();
        previous.connect_clicked(move |_| v.set_page(*v.page.borrow() - 1));
        let v = viewer.clone();
        next.connect_clicked(move |_| v.set_page(*v.page.borrow() + 1));
        let v = viewer.clone();
        zoom_in.connect_clicked(move |_| v.set_zoom((*v.zoom.borrow() * 1.2).min(6.0)));
        let v = viewer.clone();
        zoom_out.connect_clicked(move |_| v.set_zoom((*v.zoom.borrow() / 1.2).max(0.1)));
        let v = viewer.clone();
        fit.connect_clicked(move |_| v.fit_page());
        viewer.install_shortcuts();
        viewer.install_drop_target();
        viewer
    }

    fn page_count(&self) -> i32 {
        match self.document.borrow().as_ref() {
            Some(Document::Pdf(pdf)) => pdf.n_pages(),
            Some(Document::Image(_)) => 1,
            None => 0,
        }
    }

    fn page_size(&self) -> Option<(f64, f64)> {
        match self.document.borrow().as_ref()? {
            Document::Pdf(pdf) => pdf.page(*self.page.borrow()).map(|p| p.size()),
            Document::Image(image) => Some((image.width() as f64, image.height() as f64)),
        }
    }

    fn update_status(&self) {
        let count = self.page_count();
        if count == 0 {
            self.page_label.set_label("— / —");
        } else {
            self.page_label
                .set_label(&format!("{} / {count}", *self.page.borrow() + 1));
        }
        self.zoom_label.set_label(&format!(
            "{}%",
            (*self.zoom.borrow() * 100.0).round() as i32
        ));
    }

    fn set_page(&self, requested: i32) {
        let count = self.page_count();
        if count == 0 {
            return;
        }
        *self.page.borrow_mut() = requested.clamp(0, count - 1);
        self.resize_canvas();
        self.update_status();
        self.canvas.queue_draw();
    }

    fn set_zoom(&self, zoom: f64) {
        *self.zoom.borrow_mut() = zoom;
        self.resize_canvas();
        self.update_status();
        self.canvas.queue_draw();
    }

    fn fit_page(&self) {
        let Some((width, height)) = self.page_size() else {
            return;
        };
        let available_width = self.viewport.allocated_width().max(1) as f64 - 24.0;
        let available_height = self.viewport.allocated_height().max(1) as f64 - 24.0;
        self.set_zoom(
            (available_width / width)
                .min(available_height / height)
                .clamp(0.1, 6.0),
        );
    }

    fn resize_canvas(&self) {
        if let Some((width, height)) = self.page_size() {
            let scale = *self.zoom.borrow();
            self.canvas
                .set_content_width((width * scale).ceil().clamp(1.0, 10000.0) as i32);
            self.canvas
                .set_content_height((height * scale).ceil().clamp(1.0, 10000.0) as i32);
        }
    }

    fn connect_canvas(&self) {
        let document = self.document.clone();
        let page = self.page.clone();
        let zoom = self.zoom.clone();
        self.canvas.set_draw_func(move |_, context, _, _| {
            context.scale(*zoom.borrow(), *zoom.borrow());
            match document.borrow().as_ref() {
                Some(Document::Pdf(pdf)) => {
                    if let Some(page) = pdf.page(*page.borrow()) {
                        page.render(context);
                    }
                }
                Some(Document::Image(image)) => {
                    context.set_source_pixbuf(image, 0.0, 0.0);
                    let _ = context.paint();
                }
                None => {
                    context.set_source_rgb(0.16, 0.16, 0.16);
                    let _ = context.paint();
                }
            }
        });
    }

    fn rebuild_thumbnails(&self) {
        while let Some(child) = self.sidebar.first_child() {
            self.sidebar.remove(&child);
        }
        let Some(Document::Pdf(pdf)) = self.document.borrow().clone() else {
            return;
        };
        for index in 0..pdf.n_pages() {
            let button = gtk::Button::new();
            let thumbnail = gtk::DrawingArea::new();
            thumbnail.set_content_width(112);
            thumbnail.set_content_height(144);
            let page = pdf.page(index);
            thumbnail.set_draw_func(move |_, context, width, height| {
                if let Some(page) = page.as_ref() {
                    let (page_width, page_height) = page.size();
                    let scale = (width as f64 / page_width).min(height as f64 / page_height);
                    context.translate(
                        (width as f64 - page_width * scale) / 2.0,
                        (height as f64 - page_height * scale) / 2.0,
                    );
                    context.scale(scale, scale);
                    page.render(context);
                }
            });
            button.set_child(Some(&thumbnail));
            let viewer = self.clone();
            button.connect_clicked(move |_| viewer.set_page(index));
            self.sidebar.append(&button);
        }
    }

    fn open_path(&self, path: PathBuf) {
        let is_pdf = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"));
        let result = if is_pdf {
            let uri = gio::File::for_path(&path).uri();
            poppler::Document::from_file(&uri, None)
                .map(Document::Pdf)
                .map_err(|error| error.to_string())
        } else {
            gdk_pixbuf::Pixbuf::from_file(&path)
                .map(Document::Image)
                .map_err(|error| error.to_string())
        };

        match result {
            Ok(document) => {
                *self.document.borrow_mut() = Some(document);
                *self.page.borrow_mut() = 0;
                *self.zoom.borrow_mut() = 1.0;
                self.sidebar_scroller.set_visible(matches!(
                    self.document.borrow().as_ref(),
                    Some(Document::Pdf(_))
                ));
                let filename = path.file_name().unwrap_or_default().to_string_lossy();
                self.window
                    .set_title(Some(&format!("{filename} — Preview Lite")));
                self.rebuild_thumbnails();
                self.resize_canvas();
                self.update_status();
                self.canvas.queue_draw();
            }
            Err(error) => self.show_error(&format!("Could not open file: {error}")),
        }
    }

    fn choose_file(&self) {
        let dialog = gtk::FileDialog::new();
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("PDFs and images"));
        for mime in [
            "application/pdf",
            "image/jpeg",
            "image/png",
            "image/webp",
            "image/gif",
            "image/tiff",
            "image/bmp",
            "image/svg+xml",
            "image/heic",
        ] {
            filter.add_mime_type(mime);
        }
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));
        let viewer = self.clone();
        dialog.open(Some(&self.window), gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    viewer.open_path(path);
                }
            }
        });
    }

    fn install_shortcuts(&self) {
        let keys = gtk::EventControllerKey::new();
        let viewer = self.clone();
        keys.connect_key_pressed(move |_, key, _, modifiers| {
            let control = modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK);
            match (control, key) {
                (true, gtk::gdk::Key::o) => {
                    viewer.choose_file();
                    glib::Propagation::Stop
                }
                (_, gtk::gdk::Key::Page_Down | gtk::gdk::Key::space) => {
                    viewer.set_page(*viewer.page.borrow() + 1);
                    glib::Propagation::Stop
                }
                (_, gtk::gdk::Key::Page_Up | gtk::gdk::Key::BackSpace) => {
                    viewer.set_page(*viewer.page.borrow() - 1);
                    glib::Propagation::Stop
                }
                (_, gtk::gdk::Key::plus | gtk::gdk::Key::equal) => {
                    viewer.set_zoom((*viewer.zoom.borrow() * 1.2).min(6.0));
                    glib::Propagation::Stop
                }
                (_, gtk::gdk::Key::minus) => {
                    viewer.set_zoom((*viewer.zoom.borrow() / 1.2).max(0.1));
                    glib::Propagation::Stop
                }
                (_, gtk::gdk::Key::_0) => {
                    viewer.set_zoom(1.0);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        self.window.add_controller(keys);
    }

    fn install_drop_target(&self) {
        let target = gtk::DropTarget::new(gio::File::static_type(), gtk::gdk::DragAction::COPY);
        let viewer = self.clone();
        target.connect_drop(move |_, value, _, _| {
            if let Ok(file) = value.get::<gio::File>() {
                if let Some(path) = file.path() {
                    viewer.open_path(path);
                    return true;
                }
            }
            false
        });
        self.window.add_controller(target);
    }

    fn show_error(&self, message: &str) {
        let dialog = gtk::AlertDialog::builder()
            .message("Unable to open file")
            .detail(message)
            .build();
        dialog.show(Some(&self.window));
    }
}

fn main() {
    let app = adw::Application::builder()
        .application_id("io.github.previewlite.PreviewLite")
        .build();
    app.connect_activate(|app| Viewer::new(app).window.present());
    app.run();
}
