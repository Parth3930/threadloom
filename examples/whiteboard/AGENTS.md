# Threadloom AI Agent Guide

## Core Architecture
- UI built via Rust macros (`threadloom! { ... }`).
- Components uppercase (`Row`, `Section`, `Text`, `Button`, `Column`, `Grid`). Standard HTML lowercase (`div`, `span`, `p`).

## Hot Reload Rules
- Modify literals (e.g. `class="mt-4"`, `row={true}`, `gap={4}`) → instant sub-second DOM patch.
- Arbitrary Tailwind (`w-[5rem]`, `mt-[3px]`) fully supported. CSS updates instantly without full reload.

## Best Practices
- Use `Row`, `Column`, `Section`, `Grid` for layout structure.
- Rely on standard Tailwind utility classes for styling.
- Minimum code, maximum effect. Avoid over-engineering components if a simple `div` or built-in layout component works.
