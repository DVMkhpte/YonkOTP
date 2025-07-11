use gtk::prelude::*;
use gtk::{glib, Application, Builder, CssProvider, Window};
use gtk::gdk::Display;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::gio;
use gtk::gio::AppInfo;
use rusqlite::{Connection, Result};
use std::collections::HashMap;

mod otp;
use otp::start_otp_generator;

mod data_filter;
use data_filter::validate_data;

mod database;
use database::{
    init_db, insert_otp_object, select_data, select_data_secret, export_to_csv,
    update_otp_object, delete_otp_object,
};

const APP_ID: &str = "org.yonkotp.main";
const UI_FILE: &str = "resources/window.ui";
const DB_FILE: &str = "database/yonkotp_data.db";
const AES_KEY: &[u8; 32] = b"01234567890123456789012345678901";

fn main() -> Result<()> {
    let app = Application::builder().application_id(APP_ID).build();
    let conn = Rc::new(Connection::open(DB_FILE)?);
    init_db(&conn)?;
    app.connect_activate(move |app| build_ui(app, conn.clone()));
    app.run();
    Ok(())
}

fn build_ui(app: &gtk::Application, conn: Rc<Connection>) {
    let builder = Builder::from_file(UI_FILE);

    let window: gtk::ApplicationWindow = builder
        .object("main_window")
        .expect("Échec du chargement de la fenêtre");

    let listbox = builder
        .object::<gtk::ListBox>("otp_list")
        .expect("Échec du chargement de la liste OTP");

    let menu_button = builder
        .object::<gtk::MenuButton>("menu_button")
        .expect("Menu button manquant");

    let menu_button_help = menu_button.clone();
    let menu_button_export = menu_button.clone();
    let menu_button_about = menu_button.clone();

    window.set_application(Some(app));
    window.set_resizable(false);

    populate_otp_list(&listbox, conn.clone(), &window);

    load_css();

    if let Some(help_button) = builder.object::<gtk::Button>("help_button") {
        help_button.connect_clicked(move |_| {
            let url = "https://github.com/DVMkhpte/YonkOTP/issues";
            if let Err(err) = AppInfo::launch_default_for_uri(url, None::<&gio::AppLaunchContext>) {
                eprintln!("Failed to open URL: {}", err);
            }
            menu_button_help.set_active(false);
        });
    }

    if let Some(export_button) = builder.object::<gtk::Button>("export_button") {
        let conn_clone = conn.clone();
        let window_clone = window.clone();

        export_button.connect_clicked(move |_| {
            let conn_in_save = conn_clone.clone();
            let file_dialog = gtk::FileDialog::new();
            file_dialog.set_title("Exporter en CSV");
            file_dialog.set_initial_name(Some("export_otp.csv"));

            file_dialog.save(Some(&window_clone), None::<&gio::Cancellable>, move |result| {
                match result {
                    Ok(file) => {
                        if let Some(path) = file.path() {
                            let file_path = path.to_string_lossy().to_string();

                            match export_to_csv(&conn_in_save, AES_KEY, &file_path) {
                                Ok(_) => {
                                    println!("Exporté avec succès : {}", file_path);
                                    if let Some(parent_dir) = std::path::Path::new(&file_path).parent() {
                                        let _ = open::that(parent_dir);
                                    }
                                }
                                Err(e) => eprintln!("Erreur d'export : {}", e),
                            }
                        }
                    }
                    Err(err) => eprintln!("Erreur sélection fichier : {}", err),
                }
            });
            menu_button_export.set_active(false);
        });
    }

    if let Some(about_button) = builder.object::<gtk::Button>("about_button") {
        about_button.connect_clicked(move |_| {
            let about_dialog = gtk::AboutDialog::new();
            about_dialog.set_program_name(Some("YonkOTP"));
            about_dialog.set_version(Some("1.0.0"));
            about_dialog.set_comments(Some("A simple OTP manager."));
            about_dialog.set_website(Some("https://github.com/DVMkhpte/YonkOTP"));
            about_dialog.set_website_label("GitHub Repository");
            about_dialog.set_authors(&["Enzo Partel | DVM_khpte et Ryane Guehria | ryatozz"]);

            about_dialog.set_visible(true);
            menu_button_about.set_active(false);
        });
    }

    let add_key_window: Window = builder
        .object("add_key_window")
        .expect("Échec du chargement de la modale");

    add_key_window.set_decorated(false);
    add_key_window.set_transient_for(Some(&window));
    add_key_window.set_modal(true);

    let add_key_window_rc = Rc::new(RefCell::new(add_key_window));

    let service_entry = builder
        .object::<gtk::Entry>("service_name_entry")
        .expect("Champ service introuvable");
    let username_mail_entry = builder
        .object::<gtk::Entry>("username_mail_entry")
        .expect("Champ username/mail introuvable");
    let secret_entry = builder
        .object::<gtk::Entry>("secret_key_entry")
        .expect("Champ secret introuvable");

    if let Some(button) = builder.object::<gtk::Button>("add_button") {
        let add_key_window_clone = add_key_window_rc.clone();
        let service_entry_clone = service_entry.clone();
        let username_mail_entry_clone = username_mail_entry.clone();
        let secret_entry_clone = secret_entry.clone();

        button.connect_clicked(move |_| {
            service_entry_clone.set_text("");
            username_mail_entry_clone.set_text("");
            secret_entry_clone.set_text("");
            add_key_window_clone.borrow().set_visible(true);
        });

        if let Some(cancel_button) = builder.object::<gtk::Button>("cancel_button") {
            let add_key_window_clone = add_key_window_rc.clone();
            cancel_button.connect_clicked(move |_| {
                add_key_window_clone.borrow().set_visible(false);
            });
        }

        if let Some(save_button) = builder.object::<gtk::Button>("save_button") {
            let add_key_window_clone = add_key_window_rc.clone();
            let service_entry_clone = service_entry.clone();
            let username_mail_entry_clone = username_mail_entry.clone();
            let secret_entry_clone = secret_entry.clone();
            let error_label = builder
                .object::<gtk::Label>("error_label")
                .expect("Échec de récupération du label d'erreur");

            let window_clone_for_save = window.clone();
            save_button.connect_clicked(move |_| {
                let service_name = service_entry_clone.text();
                let username_mail = username_mail_entry_clone.text();
                let secret_key = secret_entry_clone.text();

                match validate_data(&service_name, &username_mail, &secret_key) {
                    Ok(_) => {
                        let _ = insert_otp_object(
                            &conn,
                            service_name.as_str(),
                            username_mail.as_str(),
                            secret_key.as_str(),
                            AES_KEY,
                        );
                        populate_otp_list(&listbox, conn.clone(), &window_clone_for_save);
                        add_key_window_clone.borrow().set_visible(false);
                    }
                    Err(err) => {
                        error_label.set_text(&err);
                    }
                }
            });
        }
    }

    window.present();
}

fn populate_otp_list<W: IsA<gtk::Window>>(
    listbox: &gtk::ListBox,
    conn: Rc<Connection>,
    parent_window: &W
) {
    while let Some(child) = listbox.first_child() {
        listbox.remove(&child);
    }

    let label_map: Rc<RefCell<HashMap<i64, (gtk::Label, gtk::Label)>>> =
        Rc::new(RefCell::new(HashMap::new()));

    let toast_label = Rc::new(gtk::Label::new(Some("Copied!")));
    toast_label.add_css_class("toast-label");
    toast_label.set_visible(false);
    listbox.append(&*toast_label);

    match select_data(&conn, AES_KEY) {
        Ok(data) => {
            for (id, service, username) in data {
                match select_data_secret(&conn, AES_KEY, id) {
                    Ok(secret_key) => {
                        // On clone pour éviter le move
                        let secret_mod = secret_key.clone();
                        let rx = start_otp_generator(id, Box::leak(secret_key.into_boxed_str()));

                        let row = gtk::ListBoxRow::new();
                        let container = gtk::Box::new(gtk::Orientation::Vertical, 5);

                        let first_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                        let service_label = gtk::Label::new(
                            Some(&format!("{}: {}", service, username))
                        );
                        service_label.set_halign(gtk::Align::Start);
                        service_label.add_css_class("otp-name");

                        let timer_label = gtk::Label::new(Some("30"));
                        timer_label.set_halign(gtk::Align::End);
                        timer_label.add_css_class("otp-timer");

                        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                        spacer.set_hexpand(true);

                        first_row.append(&service_label);
                        first_row.append(&spacer);
                        first_row.append(&timer_label);

                        let otp_label = gtk::Label::new(Some("..."));
                        otp_label.set_halign(gtk::Align::Start);
                        otp_label.add_css_class("otp-code");

                        container.append(&first_row);
                        container.append(&otp_label);
                        row.set_child(Some(&container));

                        let gesture_left = gtk::GestureClick::new();
                        gesture_left.set_button(1);
                        {
                            let otp_label_for_copy = otp_label.clone();
                            let toast_label_clone = toast_label.clone();
                            gesture_left.connect_pressed(move |_gesture, _n_press, _x, _y| {
                                if let Some(display) = gtk::gdk::Display::default() {
                                    let clipboard = display.clipboard();
                                    let otp_text = otp_label_for_copy.text();
                                    clipboard.set_text(&otp_text);

                                    let toast_label_clone = toast_label_clone.clone();
                                    glib::timeout_add_seconds_local(2, move || {
                                        toast_label_clone.set_visible(false);
                                        glib::ControlFlow::Break
                                    });
                                }
                            });
                        }
                        row.add_controller(gesture_left);

                        // Menu contextuel Modifier/Supprimer
                        let gesture_right = gtk::GestureClick::new();
                        gesture_right.set_button(3);

                        let popover = gtk::Popover::new();
                        popover.set_has_arrow(false);
                        let vbox_menu = gtk::Box::new(gtk::Orientation::Vertical, 0);
                        let supprimer_button = gtk::Button::with_label("Supprimer");
                        let modifier_button = gtk::Button::with_label("Modifier");
                        vbox_menu.append(&supprimer_button);
                        vbox_menu.append(&modifier_button);
                        popover.set_child(Some(&vbox_menu));

                        row.add_controller(gesture_right.clone());

                        {
                            let popover = popover.clone();
                            let row = row.clone();
                            gesture_right.connect_pressed(move |_gesture, n_press, x, y| {
                                if n_press == 1 {
                                    let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                                    popover.set_pointing_to(Some(&rect));
                                    popover.set_parent(&row);
                                    popover.popup();
                                }
                            });
                        }

                        // SUPPRIMER
                        {
                            let conn = conn.clone();
                            let listbox = listbox.clone();
                            let parent_window = parent_window.clone();
                            let popover_for_delete = popover.clone();
                            supprimer_button.connect_clicked(move |_| {
                                popover_for_delete.popdown();

                                let dialog = Window::builder()
                                    .transient_for(parent_window.upcast_ref())
                                    .modal(true)
                                    .title("Supprimer ?")
                                    .default_width(250)
                                    .default_height(100)
                                    .build();

                                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
                                vbox.set_margin_top(10);
                                vbox.set_margin_bottom(10);
                                vbox.set_margin_start(10);
                                vbox.set_margin_end(10);

                                let label = gtk::Label::new(Some("Confirmer la suppression ?"));
                                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                                let ok_btn = gtk::Button::with_label("Supprimer");
                                let cancel_btn = gtk::Button::with_label("Annuler");
                                hbox.append(&ok_btn);
                                hbox.append(&cancel_btn);

                                vbox.append(&label);
                                vbox.append(&hbox);
                                dialog.set_child(Some(&vbox));

                                let conn_ok = conn.clone();
                                let listbox_ok = listbox.clone();
                                let parent_ok = parent_window.clone();
                                ok_btn.connect_clicked({
                                    let dialog = dialog.clone();
                                    move |_| {
                                        let _ = delete_otp_object(&conn_ok, id);
                                        dialog.close();
                                        populate_otp_list(&listbox_ok, conn_ok.clone(), &parent_ok);
                                    }
                                });
                                cancel_btn.connect_clicked({
                                    let dialog = dialog.clone();
                                    move |_| {
                                        dialog.close();
                                    }
                                });

                                dialog.present();
                            });
                        }

                        // MODIFIER
                        {
                            let conn_clone = conn.clone();
                            let listbox_clone = listbox.clone();
                            let parent_window_clone = parent_window.clone();
                            let popover_for_mod = popover.clone();
                            let service_mod = service.clone();
                            let username_mod = username.clone();
                            let secret_mod_clone = secret_mod.clone();
                            modifier_button.connect_clicked(move |_| {
                                popover_for_mod.popdown();

                                let dialog = gtk::Dialog::with_buttons(
                                    Some("Modifier OTP"),
                                    Some(parent_window_clone.upcast_ref()),
                                    gtk::DialogFlags::MODAL,
                                    &[
                                        ("Enregistrer", gtk::ResponseType::Accept),
                                        ("Annuler", gtk::ResponseType::Cancel),
                                    ],
                                );
                                let content_area = dialog.content_area();
                                let grid = gtk::Grid::new();
                                grid.set_row_spacing(10);
                                grid.set_column_spacing(10);

                                let l1 = gtk::Label::new(Some("Service:"));
                                let e1 = gtk::Entry::new();
                                e1.set_text(&service_mod);

                                let l2 = gtk::Label::new(Some("Utilisateur:"));
                                let e2 = gtk::Entry::new();
                                e2.set_text(&username_mod);

                                let l3 = gtk::Label::new(Some("Clé (readonly):"));
                                let e3 = gtk::Entry::new();
                                e3.set_text(&secret_mod_clone);
                                e3.set_editable(false);

                                grid.attach(&l1, 0, 0, 1, 1);
                                grid.attach(&e1, 1, 0, 1, 1);
                                grid.attach(&l2, 0, 1, 1, 1);
                                grid.attach(&e2, 1, 1, 1, 1);
                                grid.attach(&l3, 0, 2, 1, 1);
                                grid.attach(&e3, 1, 2, 1, 1);

                                content_area.append(&grid);

                                let conn_ok = conn_clone.clone();
                                let listbox_ok = listbox_clone.clone();
                                let parent_ok = parent_window_clone.clone();
                                let e1_ok = e1.clone();
                                let e2_ok = e2.clone();

                                dialog.connect_response(move |d, response| {
                                    if response == gtk::ResponseType::Accept {
                                        let new_service = e1_ok.text();
                                        let new_user = e2_ok.text();
                                        let _ = update_otp_object(&conn_ok, id, &new_service, &new_user, AES_KEY);
                                        populate_otp_list(&listbox_ok, conn_ok.clone(), &parent_ok);
                                    }
                                    d.close();
                                });
                                dialog.present();
                            });
                        }

                        listbox.append(&row);

                        label_map.borrow_mut().insert(
                            id,
                            (otp_label.clone(), timer_label.clone())
                        );

                        let label_map_clone = label_map.clone();
                        glib::timeout_add_seconds_local(1, move || {
                            if let Ok((id, otp, remaining)) = rx.try_recv() {
                                if let Some((otp_label, timer_label)) =
                                    label_map_clone.borrow().get(&id)
                                {
                                    otp_label.set_text(&otp);
                                    timer_label.set_text(&remaining.to_string());
                                }
                            }
                            glib::ControlFlow::Continue
                        });
                    }
                    Err(e) => {
                        eprintln!("Erreur lors du select_data_secret pour id {}: {}", id, e);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Erreur lors de la récupération des données OTP : {}", err);
        }
    }
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string("
        .title-label {
            font-size: 24px;
            font-weight: bold;
        }
        .search-entry {
            background: #444;
            color: white;
            border-radius: 8px;
            padding: 5px;
        }
        .green-button {
            background: #28a745;
            color: white;
            font-weight: bold;
            border-radius: 8px;
        }
        .green-button:hover {
            background: #218838;
        }
        .otp-name {
            font-size: 18px;
            font-weight: normal;
            padding-top: 5px;
        }
        .otp-code {
            font-size: 30px;
            font-weight: bold;
            padding-top: 5px;
        }
        .otp-timer {
            font-size: 14px;
            font-weight: bold;
            color: #bbb; 
        }
        GtkListBoxRow {
            background-color: #222;
            border-radius: 5px;
        }
        GtkSeparator {
            background-color: #555;
            min-height: 1px;
        }
        .error-text {
            color: red;
            font-size: 14px;
            font-weight: bold;
        }
        .menu-button {
            background: transparent;
            padding: 1px;
        }
        .menu-button:hover {
            background: rgba(255, 255, 255, 0.1);
        }
        .toast-label {
            color: #fff;
            background-color: rgba(0, 0, 0, 0.6);
            border-radius: 10px;
            padding: 8px 12px;
            margin: 10px;
            font-weight: bold;
            font-size: 14px;
            transition: opacity 0.3s;
        }
    ");

    let display = Display::default().expect("Impossible de récupérer l'affichage");
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
