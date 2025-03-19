use std::time::Duration;
use std::thread;
use gtk::prelude::*;
use gtk::{glib, Application, Builder, CssProvider, Window};
use gtk::gdk::Display;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::gio;
use gtk::gio::AppInfo;
use rusqlite::{Connection, Result};

mod otp;
use otp::start_otp_generator;
mod data_filter;
use data_filter::{serach_data, validate_data};

mod database;
use database::{init_db, insert_otp_object, select_data, select_data_cond};

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
        });
    }

    // Permet de garder le programme en exécution
    loop {
        thread::sleep(Duration::from_secs(10));
    }

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

                    insert_otp_object(&conn, service_name.as_str(), username_mail.as_str(), secret_key.as_str(), AES_KEY);
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

fn populate_otp_list(listbox: &gtk::ListBox, conn: Rc<Connection>) {
    // Effacer tous les enfants de la liste
    while let Some(child) = listbox.first_child() {
        listbox.remove(&child);
    }

    // Récupérer les données depuis la BDD
    match select_data(&conn, AES_KEY) {
        Ok(data) => {
            for (id, service, username) in data {
                add_otp_entry(listbox, id, &service, &username);
            }
        }
        Err(err) => {
            eprintln!("Erreur lors de la récupération des données OTP : {}", err);
        }
    }
}

fn add_otp_entry(listbox: &gtk::ListBox, id: i64, service: &str, username: &str) {
    let row = gtk::ListBoxRow::new();
    let container = gtk::Box::new(gtk::Orientation::Vertical, 5);

    // Première ligne : Service + Timer
    let first_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let service_label = gtk::Label::new(Some(&format!("{}: {}", service, username)));
    service_label.set_halign(gtk::Align::Start);
    service_label.add_css_class("otp-name");

    let timer_label = gtk::Label::new(Some("30")); // Timer (à remplacer plus tard par un vrai countdown)
    timer_label.set_halign(gtk::Align::End);
    timer_label.add_css_class("otp-timer");

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    first_row.append(&service_label);
    first_row.append(&spacer);
    first_row.append(&timer_label);

    // Deuxième ligne : Code OTP
    let otp_label = gtk::Label::new(Some("123 789"));
    otp_label.set_halign(gtk::Align::Start);
    otp_label.add_css_class("otp-code");

    // Ajouter les éléments au container principal
    container.append(&first_row);
    container.append(&otp_label);
    
    // Ajouter la structure complète à la ligne
    row.set_child(Some(&container));

    // Ajouter à la liste
    listbox.append(&row);

    // Ajouter un séparateur
    let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
    listbox.append(&separator);
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