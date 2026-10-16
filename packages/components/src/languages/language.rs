//! Semua teks antarmuka di satu tempat.
//!
//! [`Strings`] adalah struct bersarang per area (`home`, `login`, `navbar`, ...).
//! Isinya didefinisikan **sekali** oleh macro `strings!` di bawah, bersama tabel
//! bahasa Inggris [`EN`]. Menambah bahasa nanti = menulis satu literal
//! `Strings { .. }` lagi; compiler memaksa semua field terisi.
//!
//! Aturan:
//! - Teks bertanda `{nama}` adalah template; isi lewat method di bagian
//!   "Template" (bawah file), jangan `format!` manual di komponen.
//! - Pesan error disimpan sebagai tipe/enum (`CalcError`, `ApiError`, ...) dan baru
//!   diterjemahkan saat ditampilkan, supaya ganti bahasa langsung berlaku.
//! - Bukan teks UI (dan karena itu tidak ada di sini): nama perintah shell,
//!   nama ikon Material Symbols, URL, id/kunci DOM, log boot `systemd` simulasi,
//!   dan isi `data/*.json` (konten, nanti pindah ke database).

use std::fmt::Display;

/// Mengisi `{nama}` pada `template` dengan `args`, satu kali jalan (nilai yang
/// berisi `{x}` tidak diproses lagi). Placeholder tak dikenal dibiarkan apa
/// adanya; test di `languages/tests.rs` memastikan itu tidak terjadi.
pub fn fill(template: &str, args: &[(&str, &dyn Display)]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let name = &after[..end];
        match args.iter().find(|(k, _)| *k == name) {
            Some((_, v)) => {
                let _ = write!(out, "{v}");
            }
            None => {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Mendefinisikan struct grup, struct [`Strings`], tabel [`EN`], dan
/// `Strings::entries()` (daftar `(jalur, teks)` untuk test) dari satu daftar.
macro_rules! strings {
    ($(
        $group:ident : $Group:ident {
            $( $(#[$meta:meta])* $field:ident : $text:literal ),* $(,)?
        }
    ),* $(,)?) => {
        $(
            #[derive(Debug)]
            pub struct $Group {
                $( $(#[$meta])* pub $field: &'static str, )*
            }
        )*

        #[derive(Debug)]
        pub struct Strings {
            $( pub $group: $Group, )*
        }

        /// Bahasa Inggris: bahasa bawaan.
        pub static EN: Strings = Strings {
            $( $group: $Group { $( $field: $text, )* }, )*
        };

        impl Strings {
            /// Setiap teks beserta jalurnya (`"login.unlock"`), untuk test.
            pub fn entries(&self) -> Vec<(String, &'static str)> {
                let mut all = Vec::new();
                $( $(
                    all.push((
                        format!("{}.{}", stringify!($group), stringify!($field)),
                        self.$group.$field,
                    ));
                )* )*
                all
            }
        }
    };
}

strings! {
    // ------------------------------------------------------------ umum
    common: Common {
        github: "GitHub",
        discord: "Discord",
        linkedin: "LinkedIn",
    },

    // ------------------------------------------- judul tab + metadata
    meta: Meta {
        home_title: "Generation Menu",
        home_description: "A NixOS-style boot menu that links to my GitHub, LinkedIn, GitHub journal and a desktop OS simulated in the browser, built with Rust and Dioxus (WebAssembly).",
        desktop_title: "Deisktify",
        desktop_description: "Deisktify: a desktop operating system simulated in the browser, with a window manager, calculator, radio, ambient sounds, AI chat and live chat. Built with Rust and Dioxus.",
        journal_title: "GitHub Journal",
        journal_description: "A journal of my GitHub activity: pinned repositories, contribution volume, streaks, languages and stack.",
    },

    // ---------------------------------------------------- kalender
    calendar: Calendar {
        sunday: "Sunday",
        monday: "Monday",
        tuesday: "Tuesday",
        wednesday: "Wednesday",
        thursday: "Thursday",
        friday: "Friday",
        saturday: "Saturday",
        january: "January",
        february: "February",
        march: "March",
        april: "April",
        may: "May",
        june: "June",
        july: "July",
        august: "August",
        september: "September",
        october: "October",
        november: "November",
        december: "December",
        jan: "Jan",
        feb: "Feb",
        mar: "Mar",
        apr: "Apr",
        may_short: "May",
        jun: "Jun",
        jul: "Jul",
        aug: "Aug",
        sep: "Sep",
        oct: "Oct",
        nov: "Nov",
        dec: "Dec",
        am: "AM",
        pm: "PM",
    },

    // ------------------------------------------------- jam di navbar
    clock: Clock {
        switch_to_12h: "Switch to 12-hour clock",
        switch_to_24h: "Switch to 24-hour clock",
    },

    // ---------------------------------------------- waktu relatif
    time: Time {
        just_now: "just now",
        minute_one: "{n} minute ago",
        minute_other: "{n} minutes ago",
        hour_one: "{n} hour ago",
        hour_other: "{n} hours ago",
        day_one: "{n} day ago",
        day_other: "{n} days ago",
        week_one: "{n} week ago",
        week_other: "{n} weeks ago",
        month_one: "{n} month ago",
        month_other: "{n} months ago",
        year_one: "{n} year ago",
        year_other: "{n} years ago",
    },

    // ------------------------------------------------- error API
    api: Api {
        not_configured: "The server address is not configured.",
        invalid_request: "Invalid request.",
        too_large: "The request is too large.",
        unreachable: "Could not reach the server.",
        rate_limited: "Too many requests, try again later.",
        not_found: "Not found.",
        server_error: "The server had a problem, try again later.",
        rejected: "The server rejected the request.",
        bad_response: "The server sent an unexpected response.",
    },

    // ------------------------- data dari server (loading / gagal / ulangi)
    remote: Remote {
        loading: "Loading…",
        retry: "Try again",
        failed: "Could not load this content.",
    },

    // ------------------------------------------------ halaman Home
    home: Home {
        loading: "Loading boot menu…",
        failed: "The boot menu could not be loaded.",
        open_desktop: "Open the desktop",
        open_journal: "Open the GitHub journal",
        list_label: "Boot generations",
        /// `{number}`, `{build}`, `{kernel}`, `{date}`
        generation_line: "NixOS (Generation {number} 22.11.2979.47c003416{build}, Linux Kernel {kernel}, Built on {date})",
        reboot_firmware: "Reboot Into Firmware Interface",
        auto_boot_cancelled: "Auto boot cancelled",
        /// `{seconds}`
        auto_boot_countdown: "Boot in {seconds}s · any key or tap cancels",
        hint: "Use ↑ / ↓ or j / k to move · Enter or click to open",
    },

    // ------------------------------------------------ boot screen
    booting: Booting {
        skip: "Skip ⏎",
        screen_label: "System boot log. Press any key to skip.",
    },

    // ------------------------------------------------------ login
    login: Login {
        avatar_alt: "Profile",
        keyboard: "Keyboard: EN",
        password_label: "Password",
        unlock: "[ Unlock ]",
        wrong: "[ Wrong ]",
        auto_login: "logging in automatically…",
        press_enter: "press enter, or click [ Unlock ]",
        /// Tanggal palsu di layar kunci (dekorasi).
        lock_date: "Thu, 03 Sep 2026",
        /// `{name}`
        welcome_back: "Welcome back, {name}",
        access_granted: "access granted",
    },

    // ----------------------------------------------- judul aplikasi
    apps: Apps {
        calculator: "Calculator",
        about: "About",
        settings: "Settings",
        file_manager: "File Manager",
        radio: "Radio Indonesia",
        browser: "Locked Browser",
        embience: "Embience",
        live_chat: "Live Chat!",
        ai_assistant: "AI Assistant",
    },

    // ----------------------------------------------- bingkai window
    window: Window {
        minimize: "Minimize",
        maximize: "Maximize",
        close: "Close",
        workspace: "Desktop workspace",
    },

    // ----------------------------------------------------- navbar
    navbar: Navbar {
        menu_file: "File",
        menu_edit: "Edit",
        menu_view: "View",
        menu_go: "Go",
        menu_tools: "Tools",
        menu_settings: "Settings",
        /// `{item}`
        menu_hint: "Menu \"{item}\" is decorative for now, nothing inside yet.",
        about_tip: "A personal simulated desktop by Moch.",
        open_main_menu: "Open main menu",
        logo_alt: "NixDesktop logo",
        brand_desktop: "NixDesktop",
        brand_mobile: "NixMobile",
        main_navigation: "Main navigation",
        open_status_menu: "Open status menu",
        toggle_alt: "Toggle",
        system_status: "System status",
        wifi_on: "Wi-Fi on",
        wifi_off: "Wi-Fi off",
        wifi_alt: "Wi-Fi",
        sound_on: "Sound on",
        sound_muted: "Sound muted",
        volume_alt: "Volume",
        brightness_title: "Screen brightness",
        brightness_alt: "Brightness",
        notifications_on: "Notifications on",
        notifications_muted: "Notifications muted",
        notification_alt: "Notification",
        battery_title: "Battery",
        battery_alt: "Battery",
        battery_tip: "100% · Fully charged",
        switch: "Switch",
        tty: "TTY",
    },

    // ------------------------------------------------------- dock
    dock: Dock {
        social_links: "Social links",
        open_social_menu: "Open social menu",
        app_dock: "Application dock",
        open_settings: "Open settings",
        about_nixdesktop: "About NixDesktop",
        chat: "Chat",
        open_chat_menu: "Open chat menu",
        live_chat: "Live Chat",
    },

    // ------------------------------------------------- mode terminal
    terminal: Terminal {
        panel_2: "SHELL 2",
        panel_3: "SHELL 3",
        panel_4: "SHELL 4",
        panel_5: "SHELL 5",
        panel_6: "SHELL 6",
        coming_soon: "Coming Soon",
        switch: "Switch",
        desktop: "Desktop",
        no_messages: "No messages yet.",
        no_commands: "No commands run yet.",
        welcome_before: "Welcome to NixOS 26.11 (Zakar) — Dioxus terminal simulator. Type ",
        welcome_after: " to get started.",
        help_header: "Available commands:",
        help_help: "  help            show this message",
        help_pwd: "  pwd             print working directory",
        help_whoami: "  whoami          print current user",
        help_ls: "  ls              list directory contents",
        help_cd: "  cd <dir>        change directory (.. or no arg goes home)",
        help_cat: "  cat <file>      read a file (try: cat about.txt)",
        help_neofetch: "  neofetch        system info",
        help_date: "  date            current date & time",
        help_echo: "  echo <text>     print text",
        help_history: "  history         show command history",
        help_switch: "  switch          jump to Desktop Mode",
        help_clear: "  clear           clear the screen",
        neofetch_title: "NixOS 26.11 (Zakar)",
        neofetch_rule: "-------------------",
        neofetch_os: "OS:         NixOS 26.11 (Zakar)",
        neofetch_kernel: "Kernel:     Linux 6.18.38",
        neofetch_arch: "Arch:       x86_64",
        neofetch_de: "DE:         DeistifyNix (Wayland)",
        neofetch_built_in: "Built in:   Dioxus (Rust) + Tailwind",
        neofetch_design: "Design:     Black & White",
        neofetch_philosophy: "Philosophy: Declarative Desktop OS (Joke)",
        empty_dir: "(empty)",
        /// `{dir}`
        cd_no_such_dir: "cd: no such directory: {dir}",
        /// `{name}`
        cat_no_such_file: "cat: {name}: No such file or directory",
        cat_missing_operand: "cat: missing file operand",
        no_history: "(no history yet)",
        sudo_denied: "tquilla is not in the sudoers file. This incident will be reported.",
        switching: "switching to Desktop Mode...",
        /// `{cmd}`
        command_not_found: "command not found: {cmd}. Type 'help' for a list.",
    },

    // ---------------------------------------------------- About
    about: About {
        title: "NixOS 26.11",
        logo_alt: "NixOS Logo",
        label_version: "Version",
        value_version: "NixOS 26.11 (Zakar)",
        label_kernel: "Kernel",
        value_kernel: "Linux 6.18.38",
        label_architecture: "Architecture",
        value_architecture: "x86_64",
        label_desktop: "Desktop",
        value_desktop: "DeistifyNix (Wayland)",
        label_memory: "Memory",
        value_memory: "1.70 GB / 8.00 GB",
        label_philosophy: "Philosophy",
        value_philosophy: "Declarative Desktop OS (Joke)",
        label_design: "Design",
        value_design: "Black & White",
        label_built_in: "Built in",
        value_built_in: "Dioxus (Rust) + Tailwind",
        label_network: "Network",
        value_network: "100 People Connected",
        label_files_shared: "Files shared",
        value_files_shared: "10 Open files",
    },

    // -------------------------------------------------- Settings
    settings: Settings {
        section_about: "About",
        section_preferences: "Preferences",
        coming_soon: "Coming soon.",
    },

    // ----------------------------------------------- File Manager
    filemanager: FileManager {
        file_browser: "File browser",
        places: "Places",
        desktops: "Desktops",
        go_back: "Go back",
        go_forward: "Go forward",
        current_directory: "Current directory",
        folder_contents: "Folder contents",
        empty: "(empty)",
        documents: "Documents",
        downloads: "Downloads",
        musics: "Musics",
        pictures: "Pictures",
        videos: "Videos",
    },

    // ------------------------------------------------------ Radio
    radio: Radio {
        no_station: "No station",
        stations_label: "Radio stations",
        /// `{slogan}`
        slogan_line: "Radio Indonesia - {slogan}",
        controls_label: "Audio player controls",
        playback_controls: "Playback controls",
        previous_station: "Previous station",
        next_station: "Next station",
        play: "Play",
        pause: "Pause",
        volume: "Volume",
        playback_status: "Playback status",
        live: "LIVE",
        on_air: "ON AIR",
        paused: "PAUSED",
        stations_link: "Radio Stations ↗",
    },

    // -------------------------------------------------- Embience
    embience: Embience {
        label: "Embience",
        themes_label: "Available ambient sound themes",
        /// `{name}`
        select_ambience: "Select {name} ambience",
        /// `{name}`
        volume_of: "{name} volume",
        playback_controls: "Playback controls",
        toggle_volume: "Toggle volume",
        pause_ambience: "Pause ambience",
        play_ambience: "Play ambience",
        open_playlist: "Open playlist",
    },

    // ------------------------------------------------- Live Chat
    livechat: LiveChat {
        /// `{user}`
        attachment_alt: "Attachment from {user}",
        preview_alt: "Preview of the image about to be sent",
        remove_attached: "Remove attached image",
        remove_image: "Remove image",
        form_label: "Message submission form",
        upload_avatar: "Upload avatar",
        change_avatar: "Change avatar",
        your_avatar: "Your avatar",
        add_avatar: "Add avatar",
        username_label: "Username",
        username_placeholder: "Set username",
        url_label: "Website URL",
        url_placeholder: "url (optional)",
        message_label: "Message",
        message_placeholder: "Type your message here...",
        add_image_file: "Add image file",
        add_media: "Add Media",
        send: "Send",
        upload_not_image: "The file must be an image.",
        /// `{mb}`
        upload_too_large: "Image size must be at most {mb} MB.",
        upload_read_failed: "Could not read the image file.",
    },

    // ------------------------------------------------- AI chat
    ai: Ai {
        conversation: "Conversation",
        thinking: "Thinking...",
        empty_prompt: "Ask me anything — bring your own Gemini API key below.",
        code_block: "Code block",
        form_label: "Ask the AI assistant",
        add_attachments_aria: "Add files, and more",
        add_attachments_title: "Add Attachments",
        question_prompt: "What would you like to know?",
        send_question: "Send question",
        api_key_label: "API key:",
        show_key: "Show",
        hide_key: "Hide",
        api_key_info: "API Key information",
        api_key_tooltip: "Your key stays in memory for this session only and is sent straight from your browser to Google (as a request header). Never use a key you can't rotate.",
        api_key_help: "Your API key is only kept in memory for this session and hidden by default.",
        open_settings: "Open assistant settings",
        missing_key: "Enter your Gemini API key below first.",
        /// `{status}`
        http_failed: "Request failed (HTTP {status}).",
        empty_response: "(empty response)",
    },

    // ----------------------------------------------------- Browser
    browser: Browser {
        name_placeholder: "placeholder...",
        link_alt: "link",
        go_to_website: "Go to Website ↗",
    },

    // ----------------------------------------------------- Journal
    journal: Journal {
        loading: "Loading…",
        /// `{error}`
        load_failed: "Failed to load data: {error}",
        offline_snapshot: "Offline snapshot: live GitHub stats are unavailable right now, so the numbers below are sample data.",
        profile_alt: "Profile",
        /// `{n}` (nomor hari dalam setahun)
        build_number: "NO. {n}",
        repos: "Repos",
        stars: "Stars",
        readers: "Readers",
        eyebrow: "Profile / Editorial",
        headline_1: "Building the",
        headline_2: "Digital",
        headline_3: "Environment.",
        byline: "By Moch (@msalmanrafadhlih) — Developer, Indonesia.",
        quote: "\"Code is not just functionality; it is a curated collection of logic and craftsmanship designed to be read.\"",
        default_topic: "Project",
        section_label: "03. — SECTION",
        chronicle_title: "The Chronicle",
        chronicle_subtitle: "Visualizing the cadence of code. A timeline of open source contributions and linguistic proficiency.",
        total_volume: "Total Contribution Volume",
        /// `{growth}`
        growth_vs_last_year: "{growth} vs last year",
        dialects: "Dialects.",
        core_stack: "Core Stack (Detected)",
        timeline: "The Timeline.",
        current_streak: "Current Streak",
        /// `{count}`
        streak_days: "{count} Days",
        peak_activity: "Peak Activity",
        monthly_focus: "Monthly Focus",
        /// `{focus}`
        focus_text: "Current heavy activity detected in {focus}.",
        most_productive: "Most Productive",
        based_on_commits: "Based on commit history",
        /// `{count}`; di dalam grafik SVG
        chart_commits: "{count} Commits",
        published_in: "Published in Indonesia",
        tagline: "Crafting digital environments with a focus on typography and negative space.",
        built_with: "Built with Rust, Dioxus & TailwindCSS",
        /// `{year}`
        copyright: "© {year} msalmanrafadhlih. Open Source.",
        end_of_stream: "End of content stream",
    },

    // --------------------------------------------------- Kalkulator
    calc: Calc {
        title: "Calculator",
        mode_algebra: "Algebra",
        mode_trigonometry: "Trigonometry",
        mode_calculus: "Calculus",
        angle_rad: "RAD",
        angle_deg: "DEG",
        toggle_angle_title: "Toggle angle unit",
        output_title: "Expression",
        preview_title: "Live result preview",
        keypad_title: "Calculator keypad",
        true_text: "True",
        false_text: "False",
        /// Pemisah desimal pada tampilan; engine memakai `.` di dalam.
        decimal_separator: ".",
        tip_delete: "Delete",
        tip_clear_all: "Clear all",
        tip_equals: "Calculate",
        tip_power: "Power: 2^3 (use parentheses for long exponents)",
        tip_nth_root: "Nth root: root(n; x)",
        tip_fraction: "Fraction: frac(top; bottom)",
        tip_abs: "Absolute value",
        tip_log: "log(x) is base 10, or log(base; x)",
        tip_factorial: "Factorial",
        tip_square: "Square",
        tip_degree: "Degree",
        tip_diff: "Derivative at a point: diff(f; x; a)",
        tip_lim: "Two-sided limit: lim(f; x; a)",
        tip_limr: "Limit from the right: limr(f; x; a)",
        tip_liml: "Limit from the left: liml(f; x; a)",
        tip_ncr: "Combination: nCr(n; k)",
        tip_npr: "Permutation: nPr(n; k)",
        tip_sum: "Sum: sum(f; k; from; to)",
        tip_int: "Definite integral: int(f; x; a; b)",
    },

    // -------------------------------------------- error kalkulator
    calc_err: CalcErr {
        empty: "Empty expression",
        misplaced_decimal: "Misplaced decimal separator",
        invalid_number: "Invalid number",
        /// `{c}`
        unexpected_char: "Unrecognised character: {c}",
        extra_close_paren: "Unmatched closing parenthesis",
        invalid_expression: "Invalid expression",
        incomplete: "Incomplete expression",
        /// `{name}`
        needs_parens: "Function {name} needs parentheses",
        /// `{name}`
        arg_count_one: "{name} needs 1 argument (separate with ;)",
        /// `{name}`, `{lo}`
        arg_count_exact: "{name} needs {lo} arguments (separate with ;)",
        /// `{name}`, `{lo}`, `{hi}`
        arg_count_range: "{name} needs {lo}–{hi} arguments (separate with ;)",
        div_zero: "Cannot divide by zero",
        overflow: "Result is too large",
        undefined: "Result is undefined",
        /// `{c}`
        unknown_var: "Variable {c} is not defined yet",
        /// `{name}`
        unknown_function: "Unknown function {name}",
        complex_unsupported: "Complex numbers (i) are not supported yet",
        fractional_power_negative: "Fractional powers of negative numbers are not supported yet",
        comparison_top_level: "Comparisons (=, <, ≤, …) are only allowed at the top level",
        sqrt_negative: "Square root of a negative number is not supported yet",
        invalid_log_base: "Invalid logarithm base",
        /// `{name}`
        fn_undefined_here: "{name} is undefined here",
        arc_domain: "arcsin/arccos only accept values between -1 and 1",
        log_positive: "Logarithm only accepts positive numbers",
        factorial_domain: "Factorial only accepts integers ≥ 0",
        root_index: "Root index must be a non-zero integer",
        even_root_negative: "Even roots of negative numbers are not supported yet",
        comb_domain: "n and k must be integers with 0 ≤ k ≤ n",
        second_arg_variable: "The second argument must be a variable (e.g. x)",
        infinite_or_undefined: "The function gives an infinite or undefined value",
        derivative_infinity: "Derivatives at infinity are not supported",
        infinite_series: "Infinite series are not supported yet",
        sigma_bounds_integer: "Σ bounds must be integers",
        sigma_range_large: "Σ range is too large (max. 1,000,000 terms)",
        limit_undefined: "Limit is undefined",
        limit_no_converge: "Limit does not converge",
        limit_point_invalid: "Invalid limit point",
        limit_sides_differ: "Limit does not exist (left side ≠ right side)",
        integral_diverges: "Integral does not converge",
        integral_bounds_invalid: "Invalid integral bounds",
    },
}

// ---------------------------------------------------------------------------
// Template dan pemilih (bagian yang butuh argumen)
// ---------------------------------------------------------------------------

impl Calendar {
    /// `0` = Minggu (seperti `Date.getDay()`).
    pub fn weekday(&self, index: usize) -> &'static str {
        [
            self.sunday,
            self.monday,
            self.tuesday,
            self.wednesday,
            self.thursday,
            self.friday,
            self.saturday,
        ][index % 7]
    }

    /// `0` = Januari (seperti `Date.getMonth()`).
    pub fn month(&self, index: usize) -> &'static str {
        [
            self.january,
            self.february,
            self.march,
            self.april,
            self.may,
            self.june,
            self.july,
            self.august,
            self.september,
            self.october,
            self.november,
            self.december,
        ][index % 12]
    }

    pub fn month_short(&self, index: usize) -> &'static str {
        [
            self.jan,
            self.feb,
            self.mar,
            self.apr,
            self.may_short,
            self.jun,
            self.jul,
            self.aug,
            self.sep,
            self.oct,
            self.nov,
            self.dec,
        ][index % 12]
    }
}

/// Satuan waktu relatif.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeUnit {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl Time {
    /// "{n} minute(s) ago" dst. Pemilihan bentuk jamak ada di sini supaya
    /// bahasa lain bisa memakai aturan sendiri.
    pub fn ago(&self, n: u32, unit: TimeUnit) -> String {
        let one = n == 1;
        let template = match (unit, one) {
            (TimeUnit::Minute, true) => self.minute_one,
            (TimeUnit::Minute, false) => self.minute_other,
            (TimeUnit::Hour, true) => self.hour_one,
            (TimeUnit::Hour, false) => self.hour_other,
            (TimeUnit::Day, true) => self.day_one,
            (TimeUnit::Day, false) => self.day_other,
            (TimeUnit::Week, true) => self.week_one,
            (TimeUnit::Week, false) => self.week_other,
            (TimeUnit::Month, true) => self.month_one,
            (TimeUnit::Month, false) => self.month_other,
            (TimeUnit::Year, true) => self.year_one,
            (TimeUnit::Year, false) => self.year_other,
        };
        fill(template, &[("n", &n)])
    }
}

impl Home {
    pub fn generation_line(
        &self,
        number: u32,
        build: &dyn Display,
        kernel: &str,
        date: &str,
    ) -> String {
        fill(
            self.generation_line,
            &[
                ("number", &number),
                ("build", build),
                ("kernel", &kernel),
                ("date", &date),
            ],
        )
    }

    pub fn auto_boot_countdown(&self, seconds: i32) -> String {
        fill(self.auto_boot_countdown, &[("seconds", &seconds)])
    }
}

impl Login {
    pub fn welcome_back(&self, name: &str) -> String {
        fill(self.welcome_back, &[("name", &name)])
    }
}

impl Navbar {
    pub fn menu_items(&self) -> [&'static str; 6] {
        [
            self.menu_file,
            self.menu_edit,
            self.menu_view,
            self.menu_go,
            self.menu_tools,
            self.menu_settings,
        ]
    }

    pub fn menu_hint(&self, item: &str) -> String {
        fill(self.menu_hint, &[("item", &item)])
    }
}

impl Terminal {
    /// Nomor panel 2..=6 (label "SHELL n").
    pub fn panel(&self, n: u8) -> &'static str {
        match n {
            2 => self.panel_2,
            3 => self.panel_3,
            4 => self.panel_4,
            5 => self.panel_5,
            _ => self.panel_6,
        }
    }

    pub fn help_lines(&self) -> [&'static str; 13] {
        [
            self.help_header,
            self.help_help,
            self.help_pwd,
            self.help_whoami,
            self.help_ls,
            self.help_cd,
            self.help_cat,
            self.help_neofetch,
            self.help_date,
            self.help_echo,
            self.help_history,
            self.help_switch,
            self.help_clear,
        ]
    }

    pub fn neofetch_lines(&self) -> [&'static str; 9] {
        [
            self.neofetch_title,
            self.neofetch_rule,
            self.neofetch_os,
            self.neofetch_kernel,
            self.neofetch_arch,
            self.neofetch_de,
            self.neofetch_built_in,
            self.neofetch_design,
            self.neofetch_philosophy,
        ]
    }

    pub fn cd_no_such_dir(&self, dir: &str) -> String {
        fill(self.cd_no_such_dir, &[("dir", &dir)])
    }

    pub fn cat_no_such_file(&self, name: &str) -> String {
        fill(self.cat_no_such_file, &[("name", &name)])
    }

    pub fn command_not_found(&self, cmd: &str) -> String {
        fill(self.command_not_found, &[("cmd", &cmd)])
    }
}

impl About {
    /// Blok info pertama (Version .. Memory).
    pub fn system_info(&self) -> [(&'static str, &'static str); 5] {
        [
            (self.label_version, self.value_version),
            (self.label_kernel, self.value_kernel),
            (self.label_architecture, self.value_architecture),
            (self.label_desktop, self.value_desktop),
            (self.label_memory, self.value_memory),
        ]
    }

    /// Blok info kedua (belum dipakai di UI).
    pub fn extra_info(&self) -> [(&'static str, &'static str); 5] {
        [
            (self.label_philosophy, self.value_philosophy),
            (self.label_design, self.value_design),
            (self.label_built_in, self.value_built_in),
            (self.label_network, self.value_network),
            (self.label_files_shared, self.value_files_shared),
        ]
    }
}

impl Radio {
    pub fn slogan_line(&self, slogan: &str) -> String {
        fill(self.slogan_line, &[("slogan", &slogan)])
    }
}

impl Embience {
    pub fn select_ambience(&self, name: &str) -> String {
        fill(self.select_ambience, &[("name", &name)])
    }

    pub fn volume_of(&self, name: &str) -> String {
        fill(self.volume_of, &[("name", &name)])
    }
}

impl LiveChat {
    pub fn attachment_alt(&self, user: &str) -> String {
        fill(self.attachment_alt, &[("user", &user)])
    }

    pub fn upload_too_large(&self, mb: u64) -> String {
        fill(self.upload_too_large, &[("mb", &mb)])
    }
}

impl Ai {
    pub fn http_failed(&self, status: u16) -> String {
        fill(self.http_failed, &[("status", &status)])
    }
}

impl Journal {
    pub fn build_number(&self, n: u32) -> String {
        fill(self.build_number, &[("n", &n)])
    }

    pub fn load_failed(&self, error: &str) -> String {
        fill(self.load_failed, &[("error", &error)])
    }

    pub fn growth_vs_last_year(&self, growth: &str) -> String {
        fill(self.growth_vs_last_year, &[("growth", &growth)])
    }

    pub fn streak_days(&self, count: u32) -> String {
        fill(self.streak_days, &[("count", &count)])
    }

    pub fn focus_text(&self, focus: &str) -> String {
        fill(self.focus_text, &[("focus", &focus)])
    }

    pub fn chart_commits(&self, count: u32) -> String {
        fill(self.chart_commits, &[("count", &count)])
    }

    pub fn copyright(&self, year: i32) -> String {
        fill(self.copyright, &[("year", &year)])
    }
}

impl CalcErr {
    pub fn unexpected_char(&self, c: char) -> String {
        fill(self.unexpected_char, &[("c", &c)])
    }

    pub fn needs_parens(&self, name: &str) -> String {
        fill(self.needs_parens, &[("name", &name)])
    }

    pub fn arg_count(&self, name: &str, lo: usize, hi: usize) -> String {
        if lo == hi && lo == 1 {
            fill(self.arg_count_one, &[("name", &name)])
        } else if lo == hi {
            fill(self.arg_count_exact, &[("name", &name), ("lo", &lo)])
        } else {
            fill(
                self.arg_count_range,
                &[("name", &name), ("lo", &lo), ("hi", &hi)],
            )
        }
    }

    pub fn unknown_var(&self, c: char) -> String {
        fill(self.unknown_var, &[("c", &c)])
    }

    pub fn unknown_function(&self, name: &str) -> String {
        fill(self.unknown_function, &[("name", &name)])
    }

    pub fn fn_undefined_here(&self, name: &str) -> String {
        fill(self.fn_undefined_here, &[("name", &name)])
    }
}
