use gtk::{
    Align, Application, ApplicationWindow, Box, Button, CheckButton, DropDown, Entry, Image, Label,
    PolicyType, Revealer, RevealerTransitionType, ScrolledWindow, StringList, glib, prelude::*,
};
use pwhash::sha512_crypt;
use std::{
    self,
    cell::{Cell, RefCell},
    fs::{read_to_string, write},
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    rc::Rc,
    sync::{Arc, Mutex},
};

const APP_ID: &str = "org.gtk_rs.Congularity";

fn user_notification(reveal: &Revealer, note: &Label, user_box: &Box, text: &str) {
    note.set_text(text);
    reveal.set_reveal_child(true);
    let reveal_clone = reveal.clone();
    user_box.set_sensitive(false);
    let box_clone = user_box.clone();
    glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        reveal_clone.set_reveal_child(false);
        box_clone.set_sensitive(true);
        glib::ControlFlow::Break
    });
}

fn common_nix(pt: &String, ssh: bool, deskman: u32) -> std::io::Result<()> {
    let mut conf2 = read_to_string("NIX_files/common.nix")?;

    conf2 = if ssh {
        conf2.replace(" enable = false", " enable = true")
    } else {
        conf2
    };
    if deskman != 0 {
        conf2.push_str("  services.xserver.enable = true;\n  ");
        if deskman == 1 {
            conf2.push_str("services.xserver.displayManager.gdm.enable = true;\n  services.xserver.desktopManager.gnome.enable = true;");
        } else if deskman == 2 {
            conf2.push_str("services.displayManager.sddm.enable = true;\n  services.desktopManager.plasma6.enable = true;");
        } else if deskman == 3 {
            conf2.push_str("services.xserver.displayManager.lightdm.enable = true;\n  services.xserver.desktopManager.xfce.enable = true;");
        }
    }
    conf2.push_str("\n}");
    let pt1 = pt.to_owned() + "common.nix";
    write(pt1, conf2)?;

    Ok(())
}

fn kernel_nix(pt: &String, kern: u32) -> std::io::Result<()> {
    let mut conf = read_to_string("NIX_files/kernel.nix")?;
    let pt1 = pt.to_owned() + "kernel.nix";
    let new_ker = String::from(match kern {
        0 => "linuxPackages",
        1 => "linuxPackages_latest",
        2 => "linuxPackages_6_6",
        _ => "linuxPackages",
    });
    conf.push_str(&new_ker);
    conf.push_str(";\n}");
    // dbg!(&pt1);
    write(pt1, &conf)?;
    Ok(())
}

fn ensure_git_tracked(put: &std::path::Path) {
    let mut pt = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pt.push(put);
    if !pt.join(".git").exists() {
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&pt)
            .status();
    }
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&pt)
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
    for p in ["NIX_files/flake.nix", "NIX_files/users.nix"] {
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

    let build_status: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));

    let mut put = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    put.push(pt);

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
    //         .args(["nix", "build", ".#vm", "-L", "-o", "result"])
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

fn add_user(app: &Application, parent: &ApplicationWindow, users: Rc<RefCell<String>>) {
    let user_box = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(20)
        .hexpand(true)
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let user_window = ApplicationWindow::builder()
        .application(app)
        .child(&user_box)
        .modal(true)
        .transient_for(parent)
        .default_height(500)
        .default_width(500)
        .hexpand(true)
        .title("Adding users")
        .build();

    let user = Entry::builder().placeholder_text("Enter username").build();
    let pass = Entry::builder()
        .placeholder_text("Enter password")
        .visibility(false)
        .build();
    let hide_pass = CheckButton::builder()
        .label("Hide password")
        .active(true)
        .build();
    let sudo = CheckButton::builder()
        .label("Enable sudo")
        .active(false)
        .build();
    let wifi = CheckButton::builder()
        .label("Allow Wi-Fi")
        .active(false)
        .build();
    let gpu = CheckButton::builder()
        .label("Allow GPU & Audio access")
        .active(false)
        .build();
    let add = Button::builder().label("Add").build();
    let reset = Button::builder().label("Reset all users").build();
    reset.add_css_class("destructive-action");
    let notify = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(Align::Center)
        .valign(Align::End)
        .margin_bottom(20)
        .build();
    notify.add_css_class("toast");
    let note = Label::builder().build();
    note.add_css_class("toast_label");
    notify.append(&note);
    let reveal = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideUp)
        .transition_duration(300)
        .child(&notify)
        .reveal_child(false)
        .build();

    {
        user_box.append(&user);
        user_box.append(&pass);
        user_box.append(&hide_pass);
        user_box.append(&sudo);
        user_box.append(&wifi);
        user_box.append(&gpu);
        user_box.append(&add);
        user_box.append(&reset);
        user_box.append(&reveal);
    }

    let pas = pass.clone();

    hide_pass.connect_toggled(move |hp| {
        if hp.is_active() {
            pas.set_visibility(false);
        } else {
            pas.set_visibility(true);
        }
    });

    let amogus = users.clone();
    let pas = pass.clone();
    let reveal_clone = reveal.clone();
    let note_clone = note.clone();
    let box_clone = user_box.clone();

    add.connect_clicked(move |_| {
        if user.text().is_empty() {
            user_notification(
                &reveal_clone,
                &note_clone,
                &box_clone,
                "Error! Username is empty.",
            );
            return;
        } else if pas.text().is_empty() {
            user_notification(
                &reveal_clone,
                &note_clone,
                &box_clone,
                "Error! Password is empty.",
            );
            return;
        }

        let mut temp = amogus.borrow_mut();
        let mut opt = String::new();
        for i in [
            ("\"wheel\" ", sudo.is_active()),
            ("\"networkmanager\" ", wifi.is_active()),
            ("\"video, audio\" ", gpu.is_active()),
        ] {
            if i.1 {
                opt.push_str(i.0);
            }
        }

        let mut res = String::from("users.users.");
        res.push_str(&user.text().to_string());
        res.push_str(" = {\n    isNormalUser = true;\n    hashedPassword = \"");
        match sha512_crypt::hash(&pas.text().to_string()) {
            Ok(hash) => {
                res.push_str(&hash);
            }
            Err(e) => {
                user_notification(&reveal_clone, &note_clone, &box_clone, &e.to_string());
                return;
            }
        }
        res.push_str("\";\n  ");
        if !opt.is_empty() {
            res.push_str("extraGroups = [ ");
            res.push_str(&opt);
            res.push_str("];");
        }
        res.push_str("\n  };\n");
        // println!("{}", &res);

        temp.push_str(&res);
        user_notification(
            &reveal_clone,
            &note_clone,
            &box_clone,
            "User was added into list.",
        );
    });

    let amogus = users.clone();
    let reveal_clone = reveal.clone();
    let note_clone = note.clone();
    let box_clone = user_box.clone();
    reset.connect_clicked(move |_| {
        let mut temp = amogus.borrow_mut();
        user_notification(
            &reveal_clone,
            &note_clone,
            &box_clone,
            "Users was removed from list",
        );
        temp.clear();
    });

    user_window.present();
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
        // .halign(Align::Center)
        .build();

    let button_build = Button::builder()
        .label("Build!")
        .margin_start(50)
        .margin_end(50)
        .build();

    let button_add_user = Button::builder()
        .label("Add user")
        .halign(Align::End)
        .build();

    let presets = StringList::new(&["Custom", "Basic", "Office", "Gaming", "Server"]);

    let kernels = StringList::new(&[
        "Latest LTS kernel",
        "Latest Linux kernel",
        "Linux-6.6 kernel",
        "linux_ZOV_PLACEHOLDER",
    ]);

    let results = StringList::new(&[
        "Files only (flake & config)",
        "ISO",
        "Kernel Virtual Machine",
    ]);

    let desktop_manager = StringList::new(&["None", "GNOME", "KDE Plasma 6", "XFCE"]);

    let drop_preset = DropDown::builder()
        .model(&presets)
        .halign(Align::Center)
        .selected(0)
        .width_request(240)
        .build();

    let drop_kernel = DropDown::builder()
        .model(&kernels)
        .selected(0)
        .halign(Align::Center)
        .width_request(240)
        .build();

    let drop_deskman = DropDown::builder()
        .model(&desktop_manager)
        .selected(0)
        .halign(Align::Center)
        .width_request(240)
        .build();

    let drop_result = DropDown::builder()
        .model(&results)
        .selected(0)
        .halign(Align::Center)
        .width_request(240)
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

    let ssh = CheckButton::builder()
        .label("Enable SSH")
        .active(false)
        .margin_top(10)
        .halign(Align::Center)
        .build();

    let res_path = Entry::builder()
        .placeholder_text(
            "Please, enter path for result manually,\nstarting from projetct's directory",
        )
        // .hexpand(true)
        .halign(Align::Center)
        .width_chars(30)
        .build();

    {
        vbox.append(&button_add_user);
        vbox.append(&drop_preset);
        vbox.append(&drop_kernel);
        vbox.append(&drop_deskman);
        vbox.append(&drop_result);
        vbox.append(&scrollable);
        vbox.append(&ssh);
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
    let users = Rc::new(RefCell::new(String::new()));
    let amogus = users.clone();

    button_add_user.connect_clicked(move |_button_add_user| {
        add_user(&app_clone, &window_clone, amogus.clone());
    });

    let app_clone = app.clone();
    let window_clone = window.clone();
    let sel = drop_preset.selected();
    let uss = users.clone();

    button_build.connect_clicked(move |_button_build| {
        let res = drop_result.selected();
        let deskman = drop_deskman.selected();
        let ssh_chk = ssh.is_active();
        let kernel = drop_kernel.selected();
        let pt = if res_path.text().is_empty() {
            "RESULT/".to_string()
        } else {
            res_path.text().to_string()
        };

        let _ = add_nix(&pt);
        if sel == 0 {
            let _ = custom_nix(&checks, &pt);
            let _ = kernel_nix(&pt, kernel);
            let _ = common_nix(&pt, ssh_chk, deskman);
        }

        if !uss.borrow().clone().is_empty() {
            let mut meme = String::from("{ ... }: {");
            meme.push_str(&uss.borrow().clone().to_string());
            meme.push_str("}");
            // dbg!(&meme);
            let _ = write(pt.to_owned() + "users.nix", &meme);
        }

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
