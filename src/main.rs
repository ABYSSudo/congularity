use gtk::Button;
use gtk::Switch;
use gtk::prelude::*;
use gtk::{Align, Application, ApplicationWindow, Box, glib};
use std::fs;

const APP_ID: &str = "org.gtk_rs.Congularity";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create a button with label and margins

    let vbox = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_top(12)
        .margin_bottom(12)
        .spacing(12)
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let button = Button::builder()
        .label("[Confirm]")
        .margin_start(12)
        .margin_end(12)
        .build();
    let switch_1 = Switch::builder()
        .active(false)
        .margin_start(140)
        .margin_end(140)
        .build();
    let switch_2 = Switch::builder()
        .active(false)
        .margin_start(140)
        .margin_end(140)
        .build();

    // switch_1
    //     .bind_property("active", &switch_2, "active")
    //     .bidirectional()
    //     .build();

    vbox.append(&switch_1);
    vbox.append(&switch_2);
    vbox.append(&button);

    // Connect to "clicked" signal of `button`
    button.connect_clicked(move |button| {
        // Set the label to "[Confirm]" after the button has been clicked on
        button.set_label("Ok\u{1F44C}");
        let mut contents = String::new();
        if switch_1.is_active() {
            contents.push_str("test text\n");
        }
        if switch_2.is_active() {
            contents.push_str("hello world\n");
        }
        fs::write("test_file.txt", contents).unwrap();
    });

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Congularity")
        .child(&vbox)
        .default_width(400)
        .default_height(300)
        .build();

    // Present window
    window.present();
}
