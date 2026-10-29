use gtk::{
    Align, Application, ApplicationWindow, Box, Button, CheckButton, DropDown, Entry, Image, Label,
    PolicyType, ScrolledWindow, StringList, glib, prelude::*,
};
use std::{
    self,
    cell::Cell,
    fs::{read_to_string, write},
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    rc::Rc,
    sync::{Arc, Mutex},
};

const APP_ID: &str = "org.gtk_rs.Congularity";

fn ensure_git_tracked(pt: &std::path::Path) {
    if !pt.join(".git").exists() {
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(pt)
            .status();
    }

    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(pt)
        .status();
}

fn custom_nix(sets: &Vec<(CheckButton, String)>, pt: &String) -> std::io::Result<()> {
    let mut conf = read_to_string("NIX_files/packages.nix")?;

    for (b, s) in sets {
        if b.is_active() {
            conf.push_str("      ");
            conf.push_str(s);
            conf.push_str("\n");
        }
    }
    conf.push_str("     ];\n}");
    let pt2 = pt.to_string() + "packages.nix";
    write(pt2, conf)?;
    Ok(())
}

fn add_nix(pt: &String) -> std::io::Result<()> {
    for p in [
        "NIX_files/flake.nix",
        "NIX_files/users.nix",
        "NIX_files/kernel.nix",
        "NIX_files/common.nix",
    ] {
        let vp: String = p
            .split_once('/')
            .map(|(_, after)| after.to_string())
            .unwrap_or_default();
        let pt2 = pt.to_string() + &vp;
        // dbg!(pt2);
        let val = read_to_string(p)?;
        write(pt2, val)?;
    }
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

fn nix_build(
    opt: u32,
    pt: &String,
) -> std::io::Result<(
    Arc<Mutex<Option<Child>>>,
    Arc<Mutex<Option<Result<(), String>>>>,
)> {
    let slot: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    // Меняем на Arc<Mutex>
    let build_status: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));

    let mut put = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    put.push(pt);

    let _ = Command::new("ls");

    ensure_git_tracked(&put);

    // let mut nix = Command::new("sudo");
    // match opt {
    //     1 => nix
    //         .args(["nix", "build", ".#iso", "-L", "-o", "result"])
    //         .process_group(0)
    //         .current_dir(&put)
    //         .stdout(Stdio::inherit())
    //         .stderr(Stdio::inherit()),
    //     _ => nix
    //         .args(["nix", "build", ".#iso", "-L", "-o", "result"])
    //         .process_group(0)
    //         .current_dir(&put)
    //         .stdout(Stdio::inherit())
    //         .stderr(Stdio::inherit()),
    // };

    let mut nix = Command::new("nix");
    match opt {
        1 => nix
            .args(["build", ".#iso", "-L", "-o", "result"])
            .process_group(0)
            .current_dir(&put)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit()),
        _ => nix
            .args(["build", ".#vm", "-L", "-o", "result"])
            .process_group(0)
            .current_dir(&put)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit()),
    };

    let child = nix.spawn()?;
    *slot.lock().unwrap() = Some(child);

    let slot_for_wait = slot.clone();
    let status_for_wait = build_status.clone();

    std::thread::spawn(move || {
        if let Some(mut child) = slot_for_wait.lock().unwrap().take() {
            let res = match child.wait() {
                Ok(status) if status.success() => Ok(()),
                Ok(_) => Err("The build failed with an error".to_string()),
                Err(e) => Err(format!("Process error: {}", e)),
            };

            *status_for_wait.lock().unwrap() = Some(res);
        }
    });

    Ok((slot, build_status))
}

fn building_proccess(
    app: &Application,
    parent: &ApplicationWindow,
    slot: Arc<Mutex<Option<Child>>>,
    build_status: Arc<Mutex<Option<Result<(), String>>>>,
) {
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
    processing_window.present();

    let window_for_btn = processing_window.clone();

    abort_btn.connect_clicked(move |_| {
        if let Some(mut child) = slot.lock().unwrap().take() {
            let pid = child.id() as i32;
            unsafe {
                libc::kill(-pid, libc::SIGTERM);
            }
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(2));
                unsafe {
                    libc::kill(-pid, libc::SIGKILL);
                }
                let _ = child.wait();
            });
        }
        window_for_btn.close();
    });

    let is_finished = Rc::new(Cell::new(false));

    let status_label_ui = stand_by.clone();
    let abort_btn_ui = abort_btn.clone();
    let build_status_ui = build_status.clone();
    let is_finished_ui = is_finished.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if let Some(result) = build_status_ui.lock().unwrap().take() {
            match result {
                Ok(_) => {
                    status_label_ui.set_markup("Done! You may close this window");
                }
                Err(e) => {
                    status_label_ui.set_markup(&format!("ERROR!\n{}", e));
                }
            }

            abort_btn_ui.set_label("Close");
            is_finished_ui.set(true);

            return glib::ControlFlow::Break;
        }

        glib::ControlFlow::Continue
    });
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
    let sel = drop_preset.selected();

    button_build.connect_clicked(move |_button_build| {
        let res = drop_result.selected();
        let pt = if res_path.text().is_empty() {
            "RESULT/".to_string()
        } else {
            res_path.text().to_string()
        };
        if sel == 0 {
            let _ = custom_nix(&checks, &pt);
        }
        let _ = add_nix(&pt);
        if (res == 2) || (res == 1) {
            let (slot, status) = match nix_build(res, &pt) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Unable to start building: {}", e);
                    return;
                }
            };
            building_proccess(&app_clone, &window_clone, slot, status);
        }
    });

    drop_preset.connect_selected_notify(move |dp| {
        let enabled = dp.selected() == 0;
        scrollable.set_sensitive(enabled);
        button_build.set_sensitive(enabled);
    });

    window.present();
}
