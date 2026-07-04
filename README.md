# 🕵️‍♂️ IP Logger (GUI)

<img src="https://img.shields.io/badge/Language-Rust-orange" alt="Language: Rust" />
<img src="https://img.shields.io/badge/OS-Windows-blue" alt="OS: Windows" />
<img src="https://img.shields.io/badge/OS-Linux-Red" alt="OS: Linux" />
  
A highly advanced, cross-platform graphical IP and network process logger built in Rust using `egui` and `eframe`.

این ابزار یک سیستم مانیتورینگ شبکه و IP گرافیکی بسیار پیشرفته است که به صورت کراس‌پلتفرم (قابل‌اجرا روی ویندوز و لینوکس) با زبان Rust ساخته شده است. این برنامه مانند ابزارهای حرفه‌ای نظیر Wireshark یا Sysinternals کار می‌کند اما با ظاهری مدرن‌تر و استفاده‌ای بسیار ساده‌تر.

## ✨ Features | ویژگی‌ها

- 🗺️ **Live Public IP & Geolocation:** Detects your current public IP, City, Country, and ISP.
  **نمایش زنده IP و موقعیت مکانی:** تشخیص لحظه‌ای IP عمومی شما به همراه شهر، کشور و نام شرکت ارائه‌دهنده اینترنت (ISP).

- 🔔 **Desktop Notifications:** Instantly sends a system notification whenever your public IP changes (e.g., when turning a VPN on or off).
  **نوتیفیکیشن دسکتاپ:** اگر IP شما تغییر کند (مثلاً هنگام روشن یا خاموش کردن VPN)، فوراً یک پاپ‌آپ گرافیکی روی صفحه نمایش داده می‌شود.

- 🗂️ **Process-Centric Tree View:** Groups all connected IPs by the actual application/process (e.g., `chrome`, `telegram`).
  **رابط کاربری پروسه‌محور:** تمام کانکشن‌های شبکه را بر اساس برنامه‌ای که در حال استفاده از آن است (مثل کروم یا تلگرام) دسته‌بندی و مرتب می‌کند.

- 🟢/🔴 **Active vs. Historical Tracking:** Keeps a history of closed connections (marked red) and highlights currently active connections (marked green).
  **تفکیک کانکشن‌های زنده و قدیمی:** IPهای متصل با رنگ سبز و IPهای قدیمی که قطع شده‌اند با رنگ قرمز در تاریخچه هر برنامه باقی می‌مانند.

- 🌍 **On-Demand IP Intelligence:** Click the `Get Info` button next to any foreign IP to instantly fetch its City, Country, and ISP (with built-in caching to respect API limits).
  **دریافت آنی اطلاعات سرورهای مقصد:** با یک کلیک روی دکمه در جدول، می‌توانید موقعیت جغرافیایی و ISP مربوط به هر IP خارجی متصل به سیستم را دریافت کنید (همراه با سیستم کش هوشمند).

- 📋 **Click-to-Copy:** Click on any IP address in the tables to copy it instantly to your clipboard.
  **کپی سریع با یک کلیک:** تمام IPهای داخل جدول‌ها لینک‌های تعاملی هستند و با یک کلیک در حافظه سیستم کپی می‌شوند.

- 💾 **Persistent JSON Logging:** Automatically archives all historical processes and connections in `ip_history_v2.json`.
  **لاگ‌گیری دائمی:** تمام تاریخچه و پروسه‌ها به شکل ساختاریافته در یک فایل دیتابیس `JSON` ثبت می‌شوند.

- ⚙️ **Cross-platform:** Works natively on Windows & Linux.
  **پلتفرم:** اجرا بدون نیاز به پیش‌نیاز روی ویندوز و لینوکس. (برای دیدن پروسه‌های سایر کاربران در لینوکس، اجرای برنامه با دسترسی `sudo` پیشنهاد می‌شود).

---

## 🚀 How to Run | نحوه اجرا

### 🛠️ Build release binary | ساخت نسخه نهایی

```bash
cargo build --release
```

The optimized binary will be located at:  
فایل اجرایی نهایی و بهینه‌شده در مسیر زیر قرار می‌گیرد:
`target/release/IpLogger`

*(Optional) Strip debug symbols to reduce binary size | (اختیاری) برای کاهش بیشتر حجم برنامه:*
```bash
cargo rustc --release -- -C strip=symbols
```

---

## 🎨 UI Design (egui)
Designed with a minimal, dark-mode data table layout, using `egui` and `egui_extras` for high-performance rendering.

طراحی شده با تم تاریک، جداول تعاملی و پرفورمنس بسیار بالا به کمک `egui_extras`.

---

## 📄 License | لایسنس

This project is licensed under the [MIT License](LICENSE).  
این پروژه تحت لایسنس MIT منتشر شده است.

![Repo Badge](https://visitor-badge.laobi.icu/badge?page_id=null-err0r.IP-Logger-GUI) 
