/* The header of a sortable table sorts it. The column and its direction are
 * kept by the name of the table, since a refresh replaces the table with the
 * one the server sorted. A blank cell sorts last either way. */
const sorts = new Map();
const DESCENDING = "desc";
const ASCENDING = "asc";
const NUMERIC = "numeric";

function sortTable(table) {
    const column = Number(table.dataset.sorted);
    const headers = [...table.tHead.rows[0].cells];
    if (!headers[column]) {
        return;
    }

    const descending = table.dataset.order === DESCENDING;
    const numeric = headers[column].dataset.sort === NUMERIC;
    const body = table.tBodies[0];
    const rows = [...body.rows].filter((row) => row.cells.length === headers.length);
    /* A dollar figure is a number with a sign in front of it. */
    const key = (row) => row.cells[column].textContent.trim().replace(/[$,]/g, "");

    rows.sort((left, right) => {
        const first = key(left);
        const second = key(right);
        if (first === "" || second === "") {
            return first === second ? 0 : first === "" ? 1 : -1;
        }
        const order = numeric
            ? Number.parseFloat(first) - Number.parseFloat(second)
            : first.localeCompare(second);
        return descending ? -order : order;
    });
    body.append(...rows);

    for (const [index, header] of headers.entries()) {
        const arrow = header.querySelector("[data-arrow]");
        if (arrow) {
            arrow.textContent = index === column ? (descending ? "↓" : "↑") : "";
        }
    }
}

function restoreSorts() {
    for (const table of document.querySelectorAll("table[data-sortable]")) {
        const sort = sorts.get(table.dataset.sortable);
        if (!sort) {
            continue;
        }
        table.dataset.sorted = sort.column;
        table.dataset.order = sort.order;
        sortTable(table);
    }
}

document.addEventListener("click", (event) => {
    const header = event.target.closest("th[data-sort]");
    const table = header?.closest("table[data-sortable]");
    if (!table) {
        return;
    }

    const column = String([...header.parentElement.cells].indexOf(header));
    const order =
        table.dataset.sorted === column
            ? (table.dataset.order === DESCENDING ? ASCENDING : DESCENDING)
            : (header.dataset.sort === NUMERIC ? DESCENDING : ASCENDING);

    table.dataset.sorted = column;
    table.dataset.order = order;
    sorts.set(table.dataset.sortable, { column, order });
    sortTable(table);
});
