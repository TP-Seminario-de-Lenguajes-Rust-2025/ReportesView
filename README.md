# Trabajo Práctico Final – Marketplace Descentralizado

[![codecov](https://codecov.io/github/tp-seminario-de-lenguajes-rust-2025/reportesview/graph/badge.svg?token=pGdj0Fc1Hv)](https://codecov.io/github/tp-seminario-de-lenguajes-rust-2025/reportesview)
[![coverage](https://github.com/TP-Seminario-de-Lenguajes-Rust-2025/ReportesView/actions/workflows/coverage.yml/badge.svg)](https://github.com/TP-Seminario-de-Lenguajes-Rust-2025/ReportesView/actions/workflows/coverage.yml)

**Materia:** Seminario de Lenguajes – Opción Rust  
**Tecnología:** Rust + Ink! + Substrate  
**Cobertura de tests requerida:** ≥ 85%  
**Entregas:**  
- ⭕ Primera entrega obligatoria: **18 de julio**  
- ✅ Entrega final completa: **Antes de finalizar 2025**

<img width="8334" height="4167" alt="image" src="https://github.com/user-attachments/assets/9bc5857c-5349-45ab-9e2b-a3edac75840b" />

---

## 📜 Introducción

El presente trabajo práctico final tiene como objetivo integrar los conocimientos adquiridos durante el cursado de la materia **Seminario de Lenguajes – Opción Rust**, aplicando conceptos de programación en Rust orientados al desarrollo de contratos inteligentes sobre la plataforma **Substrate** utilizando el framework **Ink!**.

La consigna propone desarrollar una **plataforma descentralizada de compra-venta de productos**, inspirada en modelos como MercadoLibre, pero ejecutada completamente en un entorno blockchain. El sistema deberá dividirse en **dos contratos inteligentes**: uno encargado de gestionar la lógica principal del marketplace y otro destinado a la generación de reportes a partir de los datos públicos del primero.

El proyecto busca que el estudiante no solo practique la sintaxis y semántica de Rust, sino que también comprenda el diseño modular de contratos inteligentes, la separación de responsabilidades, la validación de roles y permisos, y la importancia de la transparencia, trazabilidad y reputación en contextos descentralizados.

Se espera que las entregas incluyan una implementación funcional, correctamente testeada, documentada y con una cobertura de pruebas mínima del 85%.

---

## Contrato 2 – `ReportesView` (solo lectura)

### Funcionalidades
- Consultar top 5 vendedores con mejor reputación.
- Consultar top 5 compradores con mejor reputación.
- Ver productos más vendidos.
- Estadísticas por categoría: total de ventas, calificación promedio.
- Cantidad de órdenes por usuario.

**Nota:** este contrato solo puede leer datos del contrato 1. No puede emitir calificaciones, modificar órdenes ni publicar productos.

---

## 📊 Requisitos generales

- ✅ Cobertura de tests ≥ 85% entre ambos contratos.
- ✅ Tests deben contemplar:
  - Flujos completos de compra y calificación.
  - Validaciones y errores esperados.
  - Permisos por rol.
- ✅ Código comentado y bien estructurado.

---

## 🌟 Entrega Final – Fin de año

Incluye:
- Toda la funcionalidad de ambos contratos.
- Reputación completa bidireccional.
- Reportes por lectura (contrato 2).
- Tests con cobertura ≥ 85%.
- Documentación técnica clara.

### Bonus (hasta +20%):
- Sistema de disputas.
- Simulación de pagos.

