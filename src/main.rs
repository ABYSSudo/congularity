use gtk::Button;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};

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
    let button = Button::builder()
        .label("[Confirm]")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Connect to "clicked" signal of `button`
    button.connect_clicked(|button| {
        // Set the label to "[Confirm]" after the button has been clicked on
        button.set_label("Ok\u{1F44C}");
    });

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Congularity")
        .child(&button)
        .build();

    // Present window
    window.present();
}
