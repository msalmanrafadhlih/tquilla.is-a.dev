use dioxus::prelude::*;

const CSS: Asset = asset!("/assets/redirect.css");

/// One line of the simulated `systemd` boot log.
///
/// `status` is `"OK"` for a green `[ OK ]` tag, or `""` for a bare
/// `Starting ...` line that hasn't finished yet.
///
/// `hold_ms` is how long the "kernel" pauses on this line before the
/// *next* one is allowed to print — bigger numbers simulate the unit
/// actually doing work (fsck, udev, NetworkManager, ...).
const BOOT_LOG: &[(&str, &str, u32)] = &[
    ("OK", "Started Apply Kernel Variables.", 40),
    ("OK", "Mounted Kernel Debug File System.", 35),
    ("OK", "Mounted Huge Pages File System.", 35),
    ("OK", "Mounted POSIX Message Queue File System.", 35),
    ("OK", "Started Read and set NIS domainname from /etc/sysconfig/network.", 45),
    ("OK", "Activated swap /dev/mapper/cl-swap.", 50),
    ("OK", "Reached target Swap.", 60),
    ("OK", "Started Remount Root and Kernel File Systems.", 80),
    ("", "Starting Flush Journal to Persistent Storage...", 60),
    ("", "Starting Load/Save Random Seed...", 50),
    ("", "Starting Create Static Device Nodes in /dev...", 70),
    ("OK", "Started Load/Save Random Seed.", 40),
    ("OK", "Started Flush Journal to Persistent Storage.", 60),
    ("OK", "Started Setup Virtual Console.", 45),
    ("OK", "Started Create Static Device Nodes in /dev.", 90),
    ("", "Starting udev Kernel Device Manager...", 320),
    ("OK", "Started udev Kernel Device Manager.", 70),
    ("OK", "Created slice system-lvm2\\x2dpvscan.slice.", 50),
    ("", "Starting LVM event activation on device 8:2...", 180),
    ("OK", "Started Monitoring of LVM2 mirrors, snapshots etc. using dmeventd or progress polling.", 60),
    ("OK", "Reached target Local File Systems (Pre).", 70),
    ("", "Starting File System Check on /dev/disk/by-uuid/0868ca58...", 520),
    ("OK", "Started LVM event activation on device 8:2.", 60),
    ("OK", "Started File System Check on /dev/disk/by-uuid/0868ca58.", 70),
    ("", "Mounting /boot...", 140),
    ("OK", "Mounted /boot.", 60),
    ("OK", "Reached target Local File Systems.", 70),
    ("", "Starting Import network configuration from initramfs...", 60),
    ("", "Starting Tell Plymouth To Write Out Runtime Data...", 50),
    ("", "Starting Restore /run/initramfs on shutdown...", 60),
    ("OK", "Started Restore /run/initramfs on shutdown.", 50),
    ("OK", "Started Tell Plymouth To Write Out Runtime Data.", 45),
    ("OK", "Started Import network configuration from initramfs.", 60),
    ("", "Starting Create Volatile Files and Directories...", 70),
    ("OK", "Started Create Volatile Files and Directories.", 55),
    ("", "Starting Security Auditing Service...", 180),
    ("OK", "Started Security Auditing Service.", 60),
    ("", "Starting Update UTMP about System Boot/Shutdown...", 55),
    ("OK", "Started Update UTMP about System Boot/Shutdown.", 60),
    ("", "Starting Network Manager...", 380),
    ("OK", "Started Network Manager.", 70),
    ("", "Starting Login Service...", 220),
    ("OK", "Started Login Service.", 60),
    ("OK", "Reached target Network.", 70),
    ("", "Starting Permit User Sessions...", 90),
    ("OK", "Started Permit User Sessions.", 60),
    ("OK", "Reached target Multi-User System.", 90),
];

/// Simulated Linux (systemd-style) boot splash screen.
///
/// Every line is already in the DOM, but each one is invisible until its
/// own CSS `animation-delay` — computed below from `hold_ms` — kicks in.
/// That turns a static log into a line-by-line "the kernel is booting"
/// effect purely with CSS, no timers/JS required.
#[component]
pub fn Booting() -> Element {
    // Small pause before the very first line, then walk through BOOT_LOG
    // accumulating each line's reveal time from the previous line's hold.
    let mut elapsed: u32 = 200;
    let timed_lines: Vec<(&'static str, &'static str, u32)> = BOOT_LOG
        .iter()
        .map(|entry| {
            let reveal_at = elapsed;
            elapsed += entry.2;
            (entry.0, entry.1, reveal_at)
        })
        .collect();
    let cursor_delay = elapsed + 150;

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }

        div { id: "boot-screen",
            div { class: "boot-scanlines" }

            div { id: "boot-console",
                for (status, text, delay) in timed_lines {
                    div {
                        key: "{text}",
                        class: "boot-line",
                        style: "animation-delay: {delay}ms;",

                        if status.is_empty() {
                            span { class: "boot-tag boot-tag--empty" }
                        } else {
                            span {
                                class: "boot-tag boot-tag--ok",
                                "[ {status} ]"
                            }
                        }
                        span { class: "boot-text", "{text}" }
                    }
                }

                span {
                    class: "boot-cursor",
                    style: "animation-delay: {cursor_delay}ms;",
                    "█"
                }
            }
        }
    }
}
