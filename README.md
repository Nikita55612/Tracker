# Tracker

Поиск раздач по торрент трекеру rutracker.org. Возможность быстро находить необходимые ресурсы для скачивания контента через P2P-сети.

![Tracker](https://github.com/Nikita55612/Tracker/blob/main/screenshots/Screenshot_1.png)

## config.json

В директории программы должен находиться файл конфигурации.

```js
{
  "base_url": "https://rutracker.org",  // Основной URL-адрес сайта RuTracker, к которому будет происходить обращение.
  "proxy_url": "https://ps1.blockme.site:443",  // URL-адрес прокси-сервера, который будет использоваться для обхода ограничений доступа или обеспечения анонимности.
  "cookie": "bb_session=0-52335687-cqygg3U3HlXLVNkKPD6R"  // Cookie для управления сессией пользователя.
}
```

## Стиль

[system.css](https://github.com/sakofchit/system.css): Библиотека CSS для создания интерфейсов, напоминающих системную ОС Apple, которая выпускалась в 1984-1991 годах.

