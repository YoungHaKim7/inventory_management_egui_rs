#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

#[derive(Clone, PartialEq, Eq)]
enum Tab {
    Inventory,
    AddItem,
    Warehousing,
    Shipping,
}

#[derive(Clone)]
struct InventoryItem {
    id: u32,
    name: String,
    sku: String,
    unit: String,
    location: String,
    quantity_on_hand: i32,
}

#[derive(Clone)]
enum TransactionType {
    Warehousing,
    Shipping,
}

#[derive(Clone)]
struct Transaction {
    date: (i32, u32, u32), // (year, month, day)
    item_id: u32,
    quantity: i32,
    note: String,
    txn_type: TransactionType,
}

struct AddItemForm {
    name: String,
    sku: String,
    unit: String,
    location: String,
    quantity_text: String,
    status: String,
}

impl Default for AddItemForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            sku: String::new(),
            unit: String::new(),
            location: String::new(),
            quantity_text: String::new(),
            status: String::new(),
        }
    }
}

struct MovementForm {
    item_index: usize,
    quantity_text: String,
    note: String,
    status: String,
}

impl Default for MovementForm {
    fn default() -> Self {
        Self {
            item_index: 0,
            quantity_text: String::new(),
            note: String::new(),
            status: String::new(),
        }
    }
}

struct MyApp {
    items: Vec<InventoryItem>,
    transactions: Vec<Transaction>,
    next_item_id: u32,
    selected_tab: Tab,

    // Date selection (calendar)
    selected_year: i32,
    selected_month: u32,
    selected_day: u32,

    // Forms
    add_item_form: AddItemForm,
    warehousing_form: MovementForm,
    shipping_form: MovementForm,

    // UI helpers
    inventory_filter: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let (year, month, day) = current_local_ymd();
        Self {
            items: Vec::new(),
            transactions: Vec::new(),
            next_item_id: 1,
            selected_tab: Tab::Inventory,
            selected_year: year,
            selected_month: month,
            selected_day: day,
            add_item_form: AddItemForm::default(),
            warehousing_form: MovementForm::default(),
            shipping_form: MovementForm::default(),
            inventory_filter: String::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Date:");
                self.date_picker_ui(ui);
                ui.separator();
                ui.selectable_value(&mut self.selected_tab, Tab::Inventory, "Inventory");
                ui.selectable_value(&mut self.selected_tab, Tab::AddItem, "Add Item");
                ui.selectable_value(&mut self.selected_tab, Tab::Warehousing, "Warehousing");
                ui.selectable_value(&mut self.selected_tab, Tab::Shipping, "Shipping");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_tab {
            Tab::Inventory => self.inventory_tab_ui(ui),
            Tab::AddItem => self.add_item_tab_ui(ui),
            Tab::Warehousing => self.movement_tab_ui(ui, true),
            Tab::Shipping => self.movement_tab_ui(ui, false),
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let message_opt = self.current_status_message();
                if let Some(message) = message_opt {
                    ui.label(message);
                } else {
                    ui.label(" "); // keep height stable
                }
            });
        });
    }
}

fn current_local_ymd() -> (i32, u32, u32) {
    // Minimal stand-in for a date; UI lets user change it anyway
    (2025, 1, 1)
}

impl MyApp {
    fn date_picker_ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("")
            .selected_text(format!("{}", self.selected_year))
            .show_ui(ui, |ui| {
                for year in (2000..=2035).rev() {
                    ui.selectable_value(&mut self.selected_year, year, year.to_string());
                }
            });
        egui::ComboBox::from_label("")
            .selected_text(format!("{:02}", self.selected_month))
            .show_ui(ui, |ui| {
                for m in 1..=12 {
                    ui.selectable_value(&mut self.selected_month, m, format!("{:02}", m));
                }
            });
        egui::ComboBox::from_label("")
            .selected_text(format!("{:02}", self.selected_day))
            .show_ui(ui, |ui| {
                for d in 1..=31 {
                    ui.selectable_value(&mut self.selected_day, d, format!("{:02}", d));
                }
            });
    }

    fn inventory_tab_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.text_edit_singleline(&mut self.inventory_filter);
        });
        ui.separator();

        let filter_lc = self.inventory_filter.to_lowercase();
        let mut rows: Vec<_> = self
            .items
            .iter()
            .filter(|item| {
                if filter_lc.is_empty() {
                    true
                } else {
                    item.name.to_lowercase().contains(&filter_lc)
                        || item.sku.to_lowercase().contains(&filter_lc)
                        || item.location.to_lowercase().contains(&filter_lc)
                }
            })
            .cloned()
            .collect();
        rows.sort_by(|a, b| a.name.cmp(&b.name));

        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("inventory_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("SKU");
                    ui.strong("Unit");
                    ui.strong("Location");
                    ui.strong("On hand");
                    ui.end_row();

                    for item in rows.iter() {
                        ui.label(&item.name);
                        ui.label(&item.sku);
                        ui.label(&item.unit);
                        ui.label(&item.location);
                        ui.monospace(item.quantity_on_hand.to_string());
                        ui.end_row();
                    }
                });
        });
    }

    fn add_item_tab_ui(&mut self, ui: &mut egui::Ui) {
        let form = &mut self.add_item_form;
        egui::Grid::new("add_item_form")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut form.name);
                ui.end_row();

                ui.label("SKU");
                ui.text_edit_singleline(&mut form.sku);
                ui.end_row();

                ui.label("Unit");
                ui.text_edit_singleline(&mut form.unit);
                ui.end_row();

                ui.label("Location");
                ui.text_edit_singleline(&mut form.location);
                ui.end_row();

                ui.label("Initial Qty");
                ui.text_edit_singleline(&mut form.quantity_text);
                ui.end_row();
            });

        if ui.button("Add Item").clicked() {
            let qty: i32 = form.quantity_text.trim().parse().unwrap_or(0);
            if form.name.trim().is_empty() || form.sku.trim().is_empty() {
                form.status = "Name and SKU are required".to_string();
            } else {
                let item = InventoryItem {
                    id: self.next_item_id,
                    name: form.name.trim().to_string(),
                    sku: form.sku.trim().to_string(),
                    unit: form.unit.trim().to_string(),
                    location: form.location.trim().to_string(),
                    quantity_on_hand: qty,
                };
                self.next_item_id += 1;
                self.items.push(item);
                form.status = "Item added".to_string();
                form.name.clear();
                form.sku.clear();
                form.unit.clear();
                form.location.clear();
                form.quantity_text.clear();
            }
        }
    }

    fn movement_tab_ui(&mut self, ui: &mut egui::Ui, is_warehousing: bool) {
        let form = if is_warehousing {
            &mut self.warehousing_form
        } else {
            &mut self.shipping_form
        };

        if self.items.is_empty() {
            ui.label("No items available. Add an item first.");
            return;
        }
        if form.item_index >= self.items.len() {
            form.item_index = 0;
        }

        ui.horizontal(|ui| {
            ui.label("Item");
            egui::ComboBox::from_label("")
                .selected_text(
                    self.items
                        .get(form.item_index)
                        .map(|i| i.name.clone())
                        .unwrap_or_else(|| "<no items>".into()),
                )
                .show_ui(ui, |ui| {
                    for (idx, item) in self.items.iter().enumerate() {
                        ui.selectable_value(&mut form.item_index, idx, item.name.clone());
                    }
                });
        });

        egui::Grid::new("movement_form")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Quantity");
                ui.text_edit_singleline(&mut form.quantity_text);
                ui.end_row();

                ui.label("Note");
                ui.text_edit_singleline(&mut form.note);
                ui.end_row();
            });

        let button_label = if is_warehousing { "Receive" } else { "Ship" };
        if ui.button(button_label).clicked() {
            let qty: i32 = form.quantity_text.trim().parse().unwrap_or(0);
            let sign = if is_warehousing { 1 } else { -1 };
            let adj = sign * qty;
            if let Some(item) = self.items.get_mut(form.item_index) {
                let new_qoh = item.quantity_on_hand + adj;
                if new_qoh < 0 {
                    form.status = "Insufficient stock".into();
                } else {
                    item.quantity_on_hand = new_qoh;
                    self.transactions.push(Transaction {
                        date: (self.selected_year, self.selected_month, self.selected_day),
                        item_id: item.id,
                        quantity: qty,
                        note: form.note.clone(),
                        txn_type: if is_warehousing {
                            TransactionType::Warehousing
                        } else {
                            TransactionType::Shipping
                        },
                    });
                    form.status = "Recorded".into();
                    form.quantity_text.clear();
                    form.note.clear();
                }
            }
        }

        ui.separator();
        ui.strong("Recent transactions");
        egui::ScrollArea::vertical().show(ui, |ui| {
            for txn in self.transactions.iter().rev().take(20) {
                let kind = match txn.txn_type {
                    TransactionType::Warehousing => "IN",
                    TransactionType::Shipping => "OUT",
                };
                let item_name = self
                    .items
                    .iter()
                    .find(|i| i.id == txn.item_id)
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| "?".into());
                ui.label(format!(
                    "{}-{:02}-{:02} [{}] {} x{} - {}",
                    txn.date.0, txn.date.1, txn.date.2, kind, item_name, txn.quantity, txn.note
                ));
            }
        });
    }
}

impl MyApp {
    fn current_status_message(&self) -> Option<String> {
        match self.selected_tab {
            Tab::AddItem => {
                if self.add_item_form.status.is_empty() {
                    None
                } else {
                    Some(self.add_item_form.status.clone())
                }
            }
            Tab::Warehousing => {
                if self.warehousing_form.status.is_empty() {
                    None
                } else {
                    Some(self.warehousing_form.status.clone())
                }
            }
            Tab::Shipping => {
                if self.shipping_form.status.is_empty() {
                    None
                } else {
                    Some(self.shipping_form.status.clone())
                }
            }
            Tab::Inventory => None,
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Inventory Manager",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}
