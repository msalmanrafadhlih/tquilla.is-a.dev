use bevy::prelude::*;

// ==========================================
// 1. STATE & RESOURCE DEFINITIONS
// ==========================================

// State untuk mengatur transisi antara Menu dan di dalam Game
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
    Victory,
}

// Data utama untuk papan Sudoku
#[derive(Resource)]
struct SudokuData {
    grid: [[u8; 9]; 9],
    initial_fixed: [[bool; 9]; 9],
    selected_cell: Option<(usize, usize)>,
}

impl Default for SudokuData {
    fn default() -> Self {
        // Contoh puzzle Sudoku (0 berarti kosong)
        // Dalam game nyata, ini bisa di-generate dengan algoritma
        let puzzle = [
            [5, 3, 0, 0, 7, 0, 0, 0, 0],
            [6, 0, 0, 1, 9, 5, 0, 0, 0],
            [0, 9, 8, 0, 0, 0, 0, 6, 0],
            [8, 0, 0, 0, 6, 0, 0, 0, 3],
            [4, 0, 0, 8, 0, 3, 0, 0, 1],
            [7, 0, 0, 0, 2, 0, 0, 0, 6],
            [0, 6, 0, 0, 0, 0, 2, 8, 0],
            [0, 0, 0, 4, 1, 9, 0, 0, 5],
            [0, 0, 0, 0, 8, 0, 0, 7, 9],
        ];

        let mut initial_fixed = [[false; 9]; 9];
        for r in 0..9 {
            for c in 0..9 {
                if puzzle[r][c] != 0 {
                    initial_fixed[r][c] = true;
                }
            }
        }

        Self {
            grid: puzzle,
            initial_fixed,
            selected_cell: None,
        }
    }
}

// Komponen penanda untuk UI
#[derive(Component)]
struct MenuUI;

#[derive(Component)]
struct GameUI;

#[derive(Component)]
struct VictoryUI;

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct BackButton;

#[derive(Component)]
struct CellUI {
    row: usize,
    col: usize,
}

#[derive(Component)]
struct CellText {
    row: usize,
    col: usize,
}

// ==========================================
// 2. MAIN APP BUILDER
// ==========================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sudoku Klasik".into(),
                resolution: (700., 800.).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<SudokuData>()
        
        // Setup awal
        .add_systems(Startup, setup_camera)
        
        // Sistem Menu
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(Update, menu_interaction.run_if(in_state(GameState::Menu)))
        .add_systems(OnExit(GameState::Menu), cleanup::<MenuUI>)
        
        // Sistem Game
        .add_systems(OnEnter(GameState::Playing), setup_game)
        .add_systems(Update, (
            cell_interaction, 
            keyboard_input, 
            update_cell_visuals,
            check_victory
        ).run_if(in_state(GameState::Playing)))
        .add_systems(OnExit(GameState::Playing), cleanup::<GameUI>)

        // Sistem Victory
        .add_systems(OnEnter(GameState::Victory), setup_victory)
        .add_systems(Update, back_button_interaction.run_if(in_state(GameState::Victory)))
        .add_systems(OnExit(GameState::Victory), cleanup::<VictoryUI>)
        
        .run();
}

// ==========================================
// 3. SYSTEMS IMPLEMENTATION
// ==========================================

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// --- FUNGSI UTILITAS UNTUK MEMBERSIHKAN UI ---
fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// --- MENU UTAMA ---
fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf"); // Fallback ke default font jika tidak ada

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::srgb(0.95, 0.95, 0.95).into(), // Latar belakang terang yang nyaman
                ..default()
            },
            MenuUI,
        ))
        .with_children(|parent| {
            // Judul
            parent.spawn(TextBundle::from_section(
                "SUDOKU",
                TextStyle {
                    font: font.clone(),
                    font_size: 80.0,
                    color: Color::srgb(0.2, 0.2, 0.2),
                },
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(50.0)),
                ..default()
            }));

            // Tombol Mulai
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(3.0)),
                        border_radius: BorderRadius::all(Val::Px(15.0)),
                        ..default()
                    },
                    border_color: Color::srgb(0.3, 0.5, 0.8).into(),
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                PlayButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Mulai Game",
                    TextStyle {
                        font,
                        font_size: 30.0,
                        color: Color::srgb(0.3, 0.5, 0.8),
                    },
                ));
            });
        });
}

fn menu_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PlayButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut sudoku_data: ResMut<SudokuData>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::srgb(0.8, 0.9, 1.0).into();
                // Reset data game sebelum mulai
                *sudoku_data = SudokuData::default();
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.9, 0.95, 1.0).into();
            }
            Interaction::None => {
                *color = Color::WHITE.into();
            }
        }
    }
}

// --- TAMPILAN GAME (PAPAN SUDOKU) ---
fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>, sudoku_data: Res<SudokuData>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    // Container Utama
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::srgb(0.92, 0.92, 0.92).into(),
                ..default()
            },
            GameUI,
        ))
        .with_children(|parent| {
            // Instruksi
            parent.spawn(TextBundle::from_section(
                "Pilih kotak dan ketik angka (1-9)",
                TextStyle {
                    font: font.clone(),
                    font_size: 20.0,
                    color: Color::srgb(0.4, 0.4, 0.4),
                },
            ).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), ..default() }));

            // Papan Sudoku (Grid 3x3 untuk Box Besar)
            parent.spawn(NodeBundle {
                style: Style {
                    display: Display::Grid,
                    grid_template_columns: vec![RepeatedGridTrack::flex(3, 1.0)],
                    grid_template_rows: vec![RepeatedGridTrack::flex(3, 1.0)],
                    column_gap: Val::Px(4.0), // Garis tebal antar blok 3x3
                    row_gap: Val::Px(4.0),
                    background_color: Color::srgb(0.2, 0.2, 0.2), // Warna garis tebal (hitam pekat)
                    padding: UiRect::all(Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    width: Val::Px(540.0),
                    height: Val::Px(540.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|board| {
                // Generate 9 Box (Blok 3x3)
                for box_row in 0..3 {
                    for box_col in 0..3 {
                        // Grid 3x3 kecil untuk sel dalam box
                        board.spawn(NodeBundle {
                            style: Style {
                                display: Display::Grid,
                                grid_template_columns: vec![RepeatedGridTrack::flex(3, 1.0)],
                                grid_template_rows: vec![RepeatedGridTrack::flex(3, 1.0)],
                                column_gap: Val::Px(1.0), // Garis tipis antar sel
                                row_gap: Val::Px(1.0),
                                background_color: Color::srgb(0.7, 0.7, 0.7), // Warna garis tipis
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|small_box| {
                            for r in 0..3 {
                                for c in 0..3 {
                                    let global_row = box_row * 3 + r;
                                    let global_col = box_col * 3 + c;
                                    let val = sudoku_data.grid[global_row][global_col];
                                    let text_str = if val == 0 { String::new() } else { val.to_string() };

                                    // Sel berupa tombol yang bisa diklik
                                    small_box.spawn((
                                        ButtonBundle {
                                            style: Style {
                                                width: Val::Percent(100.0),
                                                height: Val::Percent(100.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            background_color: Color::WHITE.into(),
                                            ..default()
                                        },
                                        CellUI { row: global_row, col: global_col },
                                    ))
                                    .with_children(|cell_parent| {
                                        cell_parent.spawn((
                                            TextBundle::from_section(
                                                text_str,
                                                TextStyle {
                                                    font: font.clone(),
                                                    font_size: 35.0,
                                                    color: if sudoku_data.initial_fixed[global_row][global_col] {
                                                        Color::BLACK
                                                    } else {
                                                        Color::srgb(0.2, 0.4, 0.8) // Warna biru untuk angka inputan pemain
                                                    },
                                                },
                                            ),
                                            CellText { row: global_row, col: global_col },
                                        ));
                                    });
                                }
                            }
                        });
                    }
                }
            });
            
            // Tombol Kembali
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        margin: UiRect::top(Val::Px(30.0)),
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    background_color: Color::srgb(0.8, 0.4, 0.4).into(),
                    ..default()
                },
                BackButton,
            )).with_children(|btn| {
                btn.spawn(TextBundle::from_section("Kembali ke Menu", TextStyle { font: font.clone(), font_size: 20.0, color: Color::WHITE }));
            });
        });
}

// Menangani klik pada sel grid dan tombol kembali
fn cell_interaction(
    mut interaction_query: Query<
        (&Interaction, &CellUI, &mut BackgroundColor),
        (Changed<Interaction>, With<CellUI>),
    >,
    mut back_query: Query<
        &Interaction,
        (Changed<Interaction>, With<BackButton>, Without<CellUI>),
    >,
    mut sudoku_data: ResMut<SudokuData>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Cek Tombol Kembali
    for interaction in &mut back_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Menu);
            return;
        }
    }

    // Cek Klik Sel
    for (interaction, cell, _) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Jangan pilih jika cell tersebut adalah bawaan/fixed
            if !sudoku_data.initial_fixed[cell.row][cell.col] {
                sudoku_data.selected_cell = Some((cell.row, cell.col));
            } else {
                sudoku_data.selected_cell = None;
            }
        }
    }
}

// Menangani input keyboard (Angka 1-9 & Delete)
fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sudoku_data: ResMut<SudokuData>,
) {
    if let Some((r, c)) = sudoku_data.selected_cell {
        let mut input_val = None;
        
        if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) { input_val = Some(1); }
        if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) { input_val = Some(2); }
        if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) { input_val = Some(3); }
        if keyboard.just_pressed(KeyCode::Digit4) || keyboard.just_pressed(KeyCode::Numpad4) { input_val = Some(4); }
        if keyboard.just_pressed(KeyCode::Digit5) || keyboard.just_pressed(KeyCode::Numpad5) { input_val = Some(5); }
        if keyboard.just_pressed(KeyCode::Digit6) || keyboard.just_pressed(KeyCode::Numpad6) { input_val = Some(6); }
        if keyboard.just_pressed(KeyCode::Digit7) || keyboard.just_pressed(KeyCode::Numpad7) { input_val = Some(7); }
        if keyboard.just_pressed(KeyCode::Digit8) || keyboard.just_pressed(KeyCode::Numpad8) { input_val = Some(8); }
        if keyboard.just_pressed(KeyCode::Digit9) || keyboard.just_pressed(KeyCode::Numpad9) { input_val = Some(9); }
        if keyboard.just_pressed(KeyCode::Backspace) || keyboard.just_pressed(KeyCode::Delete) { input_val = Some(0); } // 0 = Kosong

        if let Some(val) = input_val {
            sudoku_data.grid[r][c] = val;
        }
    }
}

// Logika Validasi Sudoku
fn is_valid(grid: &[[u8; 9]; 9], r: usize, c: usize) -> bool {
    let val = grid[r][c];
    if val == 0 { return true; }

    // Cek Baris & Kolom
    for i in 0..9 {
        if i != c && grid[r][i] == val { return false; }
        if i != r && grid[i][c] == val { return false; }
    }

    // Cek Box 3x3
    let box_r = (r / 3) * 3;
    let box_c = (c / 3) * 3;
    for i in 0..3 {
        for j in 0..3 {
            let cr = box_r + i;
            let cc = box_c + j;
            if (cr != r || cc != c) && grid[cr][cc] == val {
                return false;
            }
        }
    }
    true
}

// Memperbarui warna sel dan teks berdasarkan status
fn update_cell_visuals(
    sudoku_data: Res<SudokuData>,
    mut cell_query: Query<(&CellUI, &mut BackgroundColor)>,
    mut text_query: Query<(&CellText, &mut Text)>,
) {
    if !sudoku_data.is_changed() { return; }

    // Update Warna Latar Sel
    for (cell, mut bg_color) in &mut cell_query {
        let is_selected = sudoku_data.selected_cell == Some((cell.row, cell.col));
        
        if is_selected {
            *bg_color = Color::srgb(0.8, 0.9, 1.0).into(); // Biru muda untuk sel terpilih
        } else {
            *bg_color = Color::WHITE.into();
        }
    }

    // Update Teks dan Warna Error
    for (cell_text, mut text) in &mut text_query {
        let val = sudoku_data.grid[cell_text.row][cell_text.col];
        
        if val == 0 {
            text.sections[0].value = String::new();
        } else {
            text.sections[0].value = val.to_string();
            
            // Atur Warna Teks
            if sudoku_data.initial_fixed[cell_text.row][cell_text.col] {
                text.sections[0].style.color = Color::BLACK;
            } else {
                if !is_valid(&sudoku_data.grid, cell_text.row, cell_text.col) {
                    text.sections[0].style.color = Color::srgb(0.9, 0.2, 0.2); // Merah jika bentrok
                } else {
                    text.sections[0].style.color = Color::srgb(0.1, 0.5, 0.1); // Hijau/Biru jika valid input user
                }
            }
        }
    }
}

// --- LOGIKA MENANG ---
fn check_victory(
    sudoku_data: Res<SudokuData>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !sudoku_data.is_changed() { return; }

    let mut is_full = true;
    let mut is_all_valid = true;

    for r in 0..9 {
        for c in 0..9 {
            if sudoku_data.grid[r][c] == 0 {
                is_full = false;
                break;
            }
            if !is_valid(&sudoku_data.grid, r, c) {
                is_all_valid = false;
            }
        }
    }

    if is_full && is_all_valid {
        next_state.set(GameState::Victory);
    }
}

fn setup_victory(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(), // Overlay gelap
                ..default()
            },
            VictoryUI,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "SELAMAT, ANDA MENANG!",
                TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    color: Color::srgb(0.2, 0.9, 0.3),
                },
            ).with_style(Style { margin: UiRect::bottom(Val::Px(30.0)), ..default() }));

            parent.spawn((
                ButtonBundle {
                    style: Style {
                        padding: UiRect::axes(Val::Px(30.0), Val::Px(15.0)),
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                BackButton, // Pinjam komponen backbutton
            )).with_children(|btn| {
                btn.spawn(TextBundle::from_section("Kembali ke Menu", TextStyle { font, font_size: 25.0, color: Color::BLACK }));
            });
        });
}

fn back_button_interaction(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Menu);
        }
    }
}
