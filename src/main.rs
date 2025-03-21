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
use database::{init_db, insert_otp_object, select_data, select_data_secret, export_to_csv};

const APP_ID: &str = "org.yonkotp.main";
const UI_FILE: &str = "resources/window.ui";
const DB_FILE: &str = "database/yonkotp_data.db";
const AES_KEY: &[u8; 32] = b"01234567890123456789012345678901";




fn main() -> Result<()> {
    // Crée l'application GTK
    let app = Application::builder().application_id(APP_ID).build();

    // Connexion à la base SQLite et initialisation
    let conn = Rc::new(Connection::open(DB_FILE)?);
    init_db(&conn)?;

    // Lancer l'interface GTK en passant la connexion clonée
    app.connect_activate(move |app| build_ui(app, conn.clone()));

    app.run();
    Ok(())
}
    
fn build_ui(app: &gtk::Application, conn: Rc<Connection>) {
    
    // Charger l'interface à partir du fichier XML
    let builder = Builder::from_file(UI_FILE);
    
    // Récupérer la fenêtre principale
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

    populate_otp_list(&listbox, conn.clone());

    load_css();
    
    if let Some(help_button) = builder.object::<gtk::Button>("help_button") {
        help_button.connect_clicked(move |_| {
            let url = "https://github.com/DVMkhpte/YonkOTP/issues"; // Remplace par ton lien
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
    
            // Afficher la boîte de dialogue
            about_dialog.set_visible(true);
            menu_button_about.set_active(false);
        });
    }

    // Récupérer la fenêtre modale
    let add_key_window: Window = builder
        .object("add_key_window")
        .expect("Échec du chargement de la modale");

    // Désactiver la minimisation et maximisation en enlevant la barre de titre
    add_key_window.set_decorated(false);

    // Empêcher le redimensionnement et rendre la fenêtre modale
    add_key_window.set_transient_for(Some(&window)); // Associe la modale à la fenêtre principale
    add_key_window.set_modal(true);

    // Utilisation de Rc<RefCell<>> pour éviter les problèmes de propriété
    let add_key_window_rc = Rc::new(RefCell::new(add_key_window));

    // Récupérer les champs de texte
    let service_entry = builder
        .object::<gtk::Entry>("service_name_entry")
        .expect("Champ service introuvable");
    let username_mail_entry = builder
        .object::<gtk::Entry>("username_mail_entry")
        .expect("Champ username/mail introuvable");
    let secret_entry = builder
        .object::<gtk::Entry>("secret_key_entry")
        .expect("Champ secret introuvable");

    // Récupérer le bouton "Add" et afficher la modale quand on clique dessus
    if let Some(button) = builder.object::<gtk::Button>("add_button") {
        let add_key_window_clone = add_key_window_rc.clone();
        
        let service_entry_clone = service_entry.clone();
        let username_mail_entry_clone = username_mail_entry.clone();
        let secret_entry_clone = secret_entry.clone();

        button.connect_clicked(move |_| {
            // Réinitialiser les champs avant d'afficher la fenêtre
            service_entry_clone.set_text("");
            username_mail_entry_clone.set_text("");
            secret_entry_clone.set_text("");

            add_key_window_clone.borrow().set_visible(true);
        });


    // Bouton "Cancel" pour fermer la modale
    if let Some(cancel_button) = builder.object::<gtk::Button>("cancel_button") {
        let add_key_window_clone = add_key_window_rc.clone();
        cancel_button.connect_clicked(move |_| {
            add_key_window_clone.borrow().set_visible(false);
        });
    }

    // Bouton "Save" pour récupérer les inputs et fermer la modale
    if let Some(save_button) = builder.object::<gtk::Button>("save_button") {
        let add_key_window_clone = add_key_window_rc.clone();
        
        let service_entry_clone = service_entry.clone();
        let username_mail_entry_clone = username_mail_entry.clone();
        let secret_entry_clone = secret_entry.clone();

        let error_label = builder
        .object::<gtk::Label>("error_label")
        .expect("Échec de récupération du label d'erreur");

        save_button.connect_clicked(move |_| {
            let service_name = service_entry_clone.text();
            let username_mail = username_mail_entry_clone.text();
            let secret_key = secret_entry_clone.text();

            match validate_data(&service_name, &username_mail, &secret_key) {
                Ok(_) => {
                    println!("Données valides : Service: {}, Username/Mail: {}, Key: {}", 
                             service_name, username_mail, secret_key);

                    let _ = insert_otp_object(&conn, service_name.as_str(), username_mail.as_str(), secret_key.as_str(), AES_KEY);
                    // Ajouter à la liste OTP
                    populate_otp_list(&listbox, conn.clone());
    
                    add_key_window_clone.borrow().set_visible(false);
                }
                Err(err) => {
                    println!("Erreur de validation : {}", err);
                    error_label.set_text(&err);
                    // TODO: Afficher une alerte GTK (message d'erreur)
                }
            }
        });
    }

    window.present();
}
}

fn populate_otp_list(listbox: &gtk::ListBox, conn: Rc<Connection>) {

    while let Some(child) = listbox.first_child() {
        listbox.remove(&child);
    }

    let label_map: Rc<RefCell<HashMap<i64, (gtk::Label, gtk::Label)>>> = Rc::new(RefCell::new(HashMap::new()));

    match select_data(&conn, AES_KEY) {
        Ok(data) => {
            for (id, service, username) in data {
                if let Ok(secret_key) = select_data_secret(&conn, AES_KEY, id) {
                    let secret_key = secret_key.clone();
                    let rx = start_otp_generator(id, Box::leak(secret_key.into_boxed_str()));

                    let row = gtk::ListBoxRow::new();
                    let container = gtk::Box::new(gtk::Orientation::Vertical, 5);

                    let first_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                    let service_label = gtk::Label::new(Some(&format!("{}: {}", service, username)));
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
                    listbox.append(&row);

                    // Stocker les labels dans la HashMap
                    label_map.borrow_mut().insert(id, (otp_label.clone(), timer_label.clone()));

                    // Mise à jour continue via timeout
                    let label_map_clone = label_map.clone();
                    glib::timeout_add_seconds_local(1, move || {
                        if let Ok((id, otp, remaining)) = rx.try_recv() {
                            if let Some((otp_label, timer_label)) = label_map_clone.borrow().get(&id) {
                                otp_label.set_text(&otp);
                                timer_label.set_text(&remaining.to_string());
                            }
                        }
                        glib::ControlFlow::Continue
                    });
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
    provider
        .load_from_string("
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
    ");

    let display = Display::default().expect("Impossible de récupérer l'affichage");
    
    // Utilisation de la nouvelle fonction recommandée
    gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}