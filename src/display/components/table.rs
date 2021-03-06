use ::std::collections::{BTreeMap, HashMap};

use ::tui::backend::Backend;
use ::tui::layout::Rect;
use ::tui::style::{Color, Style};
use ::tui::terminal::Frame;
use ::tui::widgets::{Block, Borders, Row, Widget};

use crate::display::{Bandwidth, DisplayBandwidth, UIState};
use crate::network::{display_connection_string, display_ip_or_host};

use ::std::net::IpAddr;
use std::iter::FromIterator;

fn display_upload_and_download(bandwidth: &impl Bandwidth) -> String {
    format!(
        "{} / {}",
        DisplayBandwidth(bandwidth.get_total_bytes_uploaded() as f64),
        DisplayBandwidth(bandwidth.get_total_bytes_downloaded() as f64)
    )
}

fn sort_by_bandwidth<'a, T>(
    list: &'a mut Vec<(T, &impl Bandwidth)>,
) -> &'a Vec<(T, &'a impl Bandwidth)> {
    list.sort_by(|(_, a), (_, b)| {
        let a_highest = if a.get_total_bytes_downloaded() > a.get_total_bytes_uploaded() {
            a.get_total_bytes_downloaded()
        } else {
            a.get_total_bytes_uploaded()
        };
        let b_highest = if b.get_total_bytes_downloaded() > b.get_total_bytes_uploaded() {
            b.get_total_bytes_downloaded()
        } else {
            b.get_total_bytes_uploaded()
        };
        b_highest.cmp(&a_highest)
    });
    list
}

pub enum ColumnCount {
    Two,
    Three,
    Four,
}

impl ColumnCount {
    pub fn as_u16(&self) -> u16 {
        match &self {
            ColumnCount::Two => 2,
            ColumnCount::Three => 3,
            ColumnCount::Four => 4,
        }
    }
}

pub struct ColumnData {
    column_count: ColumnCount,
    column_widths: Vec<u16>,
}

pub struct Table<'a> {
    title: &'a str,
    column_names: &'a [&'a str],
    rows: Vec<Vec<String>>,
    breakpoints: BTreeMap<u16, ColumnData>,
}

fn truncate_middle(row: &str, max_length: u16) -> String {
    if row.len() as u16 > max_length {
        let first_slice = &row[0..(max_length as usize / 2) - 2];
        let second_slice = &row[(row.len() - (max_length / 2) as usize + 2)..row.len()];
        format!("{}[..]{}", first_slice, second_slice)
    } else {
        row.to_string()
    }
}

impl<'a> Table<'a> {
    pub fn create_connections_table(state: &UIState, ip_to_host: &HashMap<IpAddr, String>) -> Self {
        let mut connections_list = Vec::from_iter(&state.connections);
        sort_by_bandwidth(&mut connections_list);
        let connections_rows = connections_list
            .iter()
            .map(|(connection, connection_data)| {
                vec![
                    display_connection_string(
                        &connection,
                        &ip_to_host,
                        &connection_data.interface_name,
                    ),
                    connection_data.process_name.to_string(),
                    display_upload_and_download(*connection_data),
                ]
            })
            .collect();
        let connections_title = "Utilization by connection";
        let connections_column_names = &["Connection", "Process", "Rate Up / Down"];
        let mut breakpoints = BTreeMap::new();
        breakpoints.insert(
            0,
            ColumnData {
                column_count: ColumnCount::Two,
                column_widths: vec![20, 23],
            },
        );
        breakpoints.insert(
            70,
            ColumnData {
                column_count: ColumnCount::Three,
                column_widths: vec![30, 12, 23],
            },
        );
        breakpoints.insert(
            100,
            ColumnData {
                column_count: ColumnCount::Three,
                column_widths: vec![60, 12, 23],
            },
        );
        breakpoints.insert(
            140,
            ColumnData {
                column_count: ColumnCount::Three,
                column_widths: vec![100, 12, 23],
            },
        );
        Table {
            title: connections_title,
            column_names: connections_column_names,
            rows: connections_rows,
            breakpoints,
        }
    }
    pub fn create_processes_table(state: &UIState) -> Self {
        let mut processes_list = Vec::from_iter(&state.processes);
        sort_by_bandwidth(&mut processes_list);
        let processes_rows = processes_list
            .iter()
            .map(|(process_name, data_for_process)| {
                vec![
                    (*process_name).to_string(),
                    data_for_process.connection_count.to_string(),
                    display_upload_and_download(*data_for_process),
                ]
            })
            .collect();
        let processes_title = "Utilization by process name";
        let processes_column_names = &["Process", "Connections", "Rate Up / Down"];
        let mut breakpoints = BTreeMap::new();
        breakpoints.insert(
            0,
            ColumnData {
                column_count: ColumnCount::Two,
                column_widths: vec![12, 23],
            },
        );
        breakpoints.insert(
            50,
            ColumnData {
                column_count: ColumnCount::Three,
                column_widths: vec![12, 12, 23],
            },
        );
        breakpoints.insert(
            100,
            ColumnData {
                column_count: ColumnCount::Three,
                column_widths: vec![40, 12, 23],
            },
        );
        breakpoints.insert(
            140,
            ColumnData {
                column_count: ColumnCount::Three,
                column_widths: vec![40, 12, 23],
            },
        );
        Table {
            title: processes_title,
            column_names: processes_column_names,
            rows: processes_rows,
            breakpoints,
        }
    }
    pub fn create_remote_addresses_table(
        state: &UIState,
        ip_to_host: &HashMap<IpAddr, String>,
    ) -> Self {
        let mut remote_addresses_list = Vec::from_iter(&state.remote_addresses);
        sort_by_bandwidth(&mut remote_addresses_list);
        let remote_addresses_rows = remote_addresses_list
            .iter()
            .map(|(remote_address, data_for_remote_address)| {
                let remote_address = display_ip_or_host(**remote_address, &ip_to_host);
                vec![
                    remote_address,
                    data_for_remote_address.connection_count.to_string(),
                    display_upload_and_download(*data_for_remote_address),
                ]
            })
            .collect();
        let remote_addresses_title = "Utilization by remote address";
        let remote_addresses_column_names = &["Remote Address", "Connections", "Rate Up / Down"];
        let mut breakpoints = BTreeMap::new();
        breakpoints.insert(
            0,
            ColumnData {
                column_count: ColumnCount::Two,
                column_widths: vec![12, 23],
            },
        );
        breakpoints.insert(
            70,
            ColumnData {
                column_count: ColumnCount::Three,
                column_widths: vec![30, 12, 23],
            },
        );
        breakpoints.insert(
            100,
            ColumnData {
                column_count: ColumnCount::Three,
                column_widths: vec![60, 12, 23],
            },
        );
        breakpoints.insert(
            140,
            ColumnData {
                column_count: ColumnCount::Three,
                column_widths: vec![100, 12, 23],
            },
        );
        Table {
            title: remote_addresses_title,
            column_names: remote_addresses_column_names,
            rows: remote_addresses_rows,
            breakpoints,
        }
    }
    pub fn render(&self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let mut column_spacing: u16 = 0;
        let mut widths = &vec![];
        let mut column_count: &ColumnCount = &ColumnCount::Three;

        for (width_breakpoint, column_data) in self.breakpoints.iter() {
            if *width_breakpoint < rect.width {
                widths = &column_data.column_widths;
                column_count = &column_data.column_count;

                let total_column_width: u16 = widths.iter().sum();
                if rect.width < total_column_width - column_count.as_u16() {
                    column_spacing = 0;
                } else {
                    column_spacing = (rect.width - total_column_width) / column_count.as_u16();
                }
            }
        }

        let column_names = match column_count {
            ColumnCount::Two => {
                vec![self.column_names[0], self.column_names[2]] // always lose the middle column when needed
            }
            ColumnCount::Three => vec![
                self.column_names[0],
                self.column_names[1],
                self.column_names[2],
            ],
            ColumnCount::Four => vec![
                self.column_names[0],
                self.column_names[1],
                self.column_names[2],
                self.column_names[3],
            ],
        };

        let rows = self.rows.iter().map(|row| match column_count {
            ColumnCount::Two => vec![
                truncate_middle(&row[0], widths[0]),
                truncate_middle(&row[2], widths[1]),
            ],
            ColumnCount::Three => vec![
                truncate_middle(&row[0], widths[0]),
                truncate_middle(&row[1], widths[1]),
                truncate_middle(&row[2], widths[2]),
            ],
            ColumnCount::Four => vec![
                truncate_middle(&row[0], widths[0]),
                truncate_middle(&row[1], widths[1]),
                truncate_middle(&row[2], widths[2]),
                truncate_middle(&row[3], widths[3]),
            ],
        });

        let table_rows = rows.map(|row| Row::StyledData(row.into_iter(), Style::default()));

        ::tui::widgets::Table::new(column_names.into_iter(), table_rows)
            .block(Block::default().title(self.title).borders(Borders::ALL))
            .header_style(Style::default().fg(Color::Yellow))
            .widths(&widths[..])
            .style(Style::default())
            .column_spacing(column_spacing)
            .render(frame, rect);
    }
}
