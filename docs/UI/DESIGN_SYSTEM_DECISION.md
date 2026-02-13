# Design System Decision for RusToK

**Дата:** 2026-02-13  
**Решение:** ✅ **DSD (Design System Driven) — shadcn подход**

---

## 🎯 Краткое резюме

**Выбрали DSD вместо Atomic Design** по следующим причинам:

| Критерий | Atomic Design | DSD (shadcn) | Победитель |
|----------|---------------|--------------|------------|
| **Совместимость с экосистемой** | ❌ | ✅ `leptos-shadcn-pagination` | **DSD** |
| **Скорость разработки** | ⚠️ Много раздумий | ✅ Быстро | **DSD** |
| **Портирование компонентов** | ❌ Нужна адаптация | ✅ Copy-paste | **DSD** |
| **Структура** | ⚠️ 5 уровней | ✅ 3 папки | **DSD** |
| **Размер проекта** | ✅ Для больших | ✅ Для малых | **DSD** |
| **Tailwind** | ⚠️ | ✅ Отлично | **DSD** |

---

## 📁 Структура

```
apps/admin/src/components/
├── ui/              ← Все UI компоненты (button, input, card, etc.)
├── features/        ← Feature-specific (auth/, dashboard/, users/)
├── layouts/         ← Layout компоненты (sidebar, header)
└── shared/          ← Shared utilities (protected_route, error_boundary)
```

**Принцип:** Flat > Deep, Copy-paste friendly, Variants over composition

---

## 🔄 Migration Plan

### ✅ Phase 1: Рефакторинг (Next PR)
- [ ] `components/ui/button.rs` с вариантами
- [ ] `components/ui/input.rs` с вариантами
- [ ] `components/ui/card.rs` (композиция)
- [ ] `components/ui/label.rs`
- [ ] Переместить `page_header.rs` → `features/dashboard/`
- [ ] Переместить `stats_card.rs` → `features/dashboard/`

### ⬜ Phase 2: Новые компоненты
- [ ] `alert.rs`, `badge.rs`, `separator.rs`
- [ ] `table.rs`, `skeleton.rs`, `dropdown.rs`
- [ ] `dialog.rs`, `tabs.rs`, `checkbox.rs`

### ⬜ Phase 3: Портирование shadcn
- [ ] Form components (textarea, switch, radio, combobox)
- [ ] Data display (data-table, calendar, chart)
- [ ] Feedback (toast, alert-dialog, progress)
- [ ] Navigation (breadcrumb, command)

---

## 📚 Дополнительно

Полный анализ: [DESIGN_SYSTEM_ANALYSIS.md](./DESIGN_SYSTEM_ANALYSIS.md)
