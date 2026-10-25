use gtk::{
    Align, Application, ApplicationWindow, Box, Button, CheckButton, DropDown, Entry, Image, Label,
    PolicyType, ScrolledWindow, StringList, glib, prelude::*,
};
use std::fs::{read_to_string, write};
use std::process::Command;

const APP_ID: &str = "org.gtk_rs.Congularity";

fn custom_nix(sets: &Vec<(CheckButton, String)>) -> std::io::Result<()> {
    let mut conf = read_to_string("NIX_files/custom/packages.nix")?;

    for (b, s) in sets {
        if b.is_active() {
            conf.push_str("      ");
            conf.push_str(s);
            conf.push_str("\n");
        }
    }
    conf.push_str("     ];\n}");

    write("RESULT/packages.nix", conf)?;

    Ok(())
}

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn nix_build() {}

fn building_proccess(app: &Application, parent: &ApplicationWindow) {
    let gif_vbox = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(20)
        .hexpand(true)
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let processing_window = ApplicationWindow::builder()
        .application(app)
        .child(&gif_vbox)
        .modal(true)
        .transient_for(parent)
        .default_height(500)
        .default_width(500)
        .hexpand(true)
        .title("Building window")
        .build();

    let abort_btn = Button::builder()
        .label("Abort!")
        .margin_start(50)
        .margin_end(50)
        .build();

    let stand_by = Label::new(Some("Building in progress. Please, stand by."));
    let cat = Image::builder()
        .file("src/mr_cat.jpg")
        .pixel_size(250)
        .hexpand(true)
        .build();
    gif_vbox.append(&stand_by);
    gif_vbox.append(&cat);
    gif_vbox.append(&abort_btn);

    let window_for_btn = processing_window.clone();

    abort_btn.connect_clicked(move |_| {
        window_for_btn.close();
    });

    processing_window.present();
}

fn build_ui(app: &Application) {
    let vbox = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .spacing(12)
        .valign(Align::Fill)
        .halign(Align::Center)
        .build();

    let button_build = Button::builder()
        .label("Build!")
        .margin_start(50)
        .margin_end(50)
        .build();

    let presets = StringList::new(&["Custom", "Basic", "Office", "Gaming", "Server"]);

    let kernels = StringList::new(&[
        "linux_latest",
        "linux_6_6",
        "linux_LTS_PLACEHOLDER",
        "linux_ZOV_PLACEHOLDER",
    ]);

    let results = StringList::new(&[
        "Files only (flake & config)",
        "ISO",
        "Kernel Virtual Machine",
        "Bootable device",
    ]);

    let drop_preset = DropDown::builder()
        .model(&presets)
        .selected(0)
        .margin_start(0)
        .margin_end(0)
        .build();

    let drop_kernel = DropDown::builder()
        .model(&kernels)
        .selected(0)
        .margin_start(0)
        .margin_end(0)
        .hexpand(true)
        .build();

    let drop_result = DropDown::builder()
        .model(&results)
        .selected(0)
        .margin_start(0)
        .margin_end(0)
        .halign(Align::Fill)
        .build();

    let scrollbox = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_top(25)
        .spacing(15)
        .margin_bottom(25)
        .halign(Align::Center)
        // .valign(Align::Fill)
        .vexpand(true)
        .sensitive(drop_preset.selected() == 0)
        .build();

    let scrollable = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        // .min_content_width(360)
        .child(&scrollbox)
        .build();

    let mut checks: Vec<(CheckButton, String)> = Vec::new();
    for pkgs in [
        "firefox",
        "lolcat",
        "vim",
        "fastfetch",
        "btop",
        "steam",
        "cowsay",
    ] {
        let butt = CheckButton::builder()
            .label(pkgs)
            .active(false)
            .halign(Align::Start)
            .build();
        checks.push((butt.clone(), pkgs.to_string()));
        scrollbox.append(&butt);
    }

    let res_path = Entry::builder()
        .placeholder_text(
            "Please, enter path for result manually,\nstarting from projetct's directory",
        )
        .hexpand(true)
        .width_chars(30)
        .build();

    {
        vbox.append(&drop_preset);
        vbox.append(&drop_kernel);
        vbox.append(&drop_result);
        vbox.append(&scrollable);
        vbox.append(&res_path);
        vbox.append(&button_build);
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Congularity")
        .child(&vbox)
        .default_width(500)
        .default_height(700)
        .build();

    let app_clone = app.clone();
    let window_clone = window.clone();
    let val = drop_preset.selected();
    button_build.connect_clicked(move |_button_build| {
        if drop_result.selected() == 1 {
            building_proccess(&app_clone, &window_clone);
        } else if val == 0 {
            let _ = custom_nix(&checks);
        }
    });

    drop_preset.connect_selected_notify(move |dp| {
        let enabled = dp.selected() == 0;
        scrollable.set_sensitive(enabled);
        button_build.set_sensitive(enabled);
    });

    window.present();
}
