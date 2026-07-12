(function () {
  "use strict";

  const renderedKey = "renderedChart";

  function parseVisual(element) {
    try {
      return JSON.parse(element.dataset.chart || "{}");
    } catch (_error) {
      return {};
    }
  }

  function pointLabel(point) {
    return point.comparison ? `${point.x} - ${point.comparison}` : point.x;
  }

  function clear(surface) {
    while (surface.firstChild) {
      surface.removeChild(surface.firstChild);
    }
  }

  function formatValue(value) {
    return Number.isFinite(value) ? value : 0;
  }

  function renderBar(surface, visual) {
    const data = (visual.points || []).map((point) => ({
      category: point.x || "",
      comparison: point.comparison || "",
      label: pointLabel(point),
      value: formatValue(point.value),
      display: point.display_value || String(point.value),
      color: point.color,
    }));
    const categories = Array.from(new Set(data.map((d) => d.category)));
    const comparisons = Array.from(new Set(data.map((d) => d.comparison).filter(Boolean)));
    const hasComparisons = comparisons.length > 0;
    const layout = hasComparisons && visual.bar_comparison_layout === "stacked" ? "stacked" : "grouped";
    const orientation = visual.bar_orientation === "vertical" ? "vertical" : "horizontal";
    const comparisonColors = new Map();
    data.forEach((datum) => {
      if (datum.comparison && datum.color && !comparisonColors.has(datum.comparison)) {
        comparisonColors.set(datum.comparison, datum.color);
      }
    });
    const fallbackColor = window.d3
      .scaleOrdinal()
      .domain(comparisons)
      .range([
        "var(--semantic-primary)",
        "var(--semantic-info)",
        "var(--semantic-success)",
        "var(--semantic-warning)",
        "var(--color-cyan)",
        "var(--color-secondary)",
      ]);
    const colorForComparison = (comparison) => comparisonColors.get(comparison) || fallbackColor(comparison);
    const color = (datum) => (hasComparisons ? colorForComparison(datum.comparison) : "var(--semantic-primary)");

    if (hasComparisons) {
      const legend = window.d3.select(surface).append("div").attr("class", "component-d3-chart__legend");
      if (visual.legend_title) {
        legend.append("p").attr("class", "component-d3-chart__legend-title").text(visual.legend_title);
      }
      const items = legend.append("div").attr("class", "component-d3-chart__legend-items");
      comparisons.forEach((comparison) => {
        const item = items.append("span").attr("class", "component-d3-chart__legend-item");
        item
          .append("span")
          .attr("class", "component-d3-chart__legend-swatch")
          .style("background-color", colorForComparison(comparison));
        item.append("span").text(comparison);
      });
    }
    const stackedTotals = new Map(categories.map((category) => [category, { negative: 0, positive: 0 }]));
    data.forEach((datum) => {
      const totals = stackedTotals.get(datum.category);
      if (datum.value < 0) {
        totals.negative += datum.value;
      } else {
        totals.positive += datum.value;
      }
    });
    const domainMin = Math.min(
      0,
      layout === "stacked"
        ? window.d3.min(Array.from(stackedTotals.values()), (totals) => totals.negative) || 0
        : window.d3.min(data, (datum) => datum.value) || 0
    );
    let domainMax = Math.max(
      0,
      layout === "stacked"
        ? window.d3.max(Array.from(stackedTotals.values()), (totals) => totals.positive) || 0
        : window.d3.max(data, (datum) => datum.value) || 0
    );
    if (domainMin === domainMax) {
      domainMax = domainMin + 1;
    }
    const width = 760;
    const height =
      orientation === "horizontal"
        ? Math.max(220, categories.length * (hasComparisons && layout === "grouped" ? 56 : 44) + 96)
        : 360;
    const margin =
      orientation === "horizontal"
        ? { top: 22, right: 78, bottom: 58, left: 190 }
        : { top: 24, right: 42, bottom: 94, left: 64 };
    const svg = window.d3
      .select(surface)
      .append("svg")
      .attr("class", "component-d3-svg component-d3-svg--bar")
      .attr("viewBox", `0 0 ${width} ${height}`);

    function appendAxisLabels() {
      if (visual.x_axis_label) {
        svg
          .append("text")
          .attr("class", "component-d3-axis-label")
          .attr("x", (margin.left + width - margin.right) / 2)
          .attr("y", height - 12)
          .attr("text-anchor", "middle")
          .text(visual.x_axis_label);
      }
      if (visual.y_axis_label) {
        svg
          .append("text")
          .attr("class", "component-d3-axis-label")
          .attr("transform", "rotate(-90)")
          .attr("x", -((margin.top + height - margin.bottom) / 2))
          .attr("y", 16)
          .attr("text-anchor", "middle")
          .text(visual.y_axis_label);
      }
    }

    if (orientation === "vertical") {
      const x = window.d3
        .scaleBand()
        .domain(categories)
        .range([margin.left, width - margin.right])
        .padding(0.24);
      const xInner = window.d3
        .scaleBand()
        .domain(comparisons.length ? comparisons : [""])
        .range([0, x.bandwidth()])
        .padding(0.12);
      const y = window.d3
        .scaleLinear()
        .domain([domainMin, domainMax])
        .nice()
        .range([height - margin.bottom, margin.top]);

      svg
        .append("line")
        .attr("class", "component-d3-zero-line")
        .attr("x1", margin.left)
        .attr("x2", width - margin.right)
        .attr("y1", y(0))
        .attr("y2", y(0));

      svg
        .append("g")
        .attr("class", "component-d3-axis")
        .attr("transform", `translate(0,${height - margin.bottom})`)
        .call(window.d3.axisBottom(x).tickSizeOuter(0))
        .selectAll("text")
        .attr("transform", "rotate(-28)")
        .attr("text-anchor", "end");
      svg
        .append("g")
        .attr("class", "component-d3-axis")
        .attr("transform", `translate(${margin.left},0)`)
        .call(window.d3.axisLeft(y).ticks(5).tickSizeOuter(0));

      if (layout === "stacked") {
        const byCategory = window.d3.group(data, (d) => d.category);
        categories.forEach((category) => {
          let negative = 0;
          let positive = 0;
          (byCategory.get(category) || []).forEach((datum) => {
            const start = datum.value < 0 ? negative : positive;
            const end = start + datum.value;
            if (datum.value < 0) {
              negative = end;
            } else {
              positive = end;
            }
            const y0 = y(start);
            const y1 = y(end);
            svg
              .append("rect")
              .attr("class", "component-d3-bar")
              .attr("x", x(category) || margin.left)
              .attr("y", Math.min(y0, y1))
              .attr("width", x.bandwidth())
              .attr("height", Math.max(1, Math.abs(y1 - y0)))
              .style("fill", color(datum))
              .append("title")
              .text(`${datum.label}: ${datum.display}`);
          });
        });
      } else {
        svg
          .append("g")
          .selectAll("rect")
          .data(data)
          .join("rect")
          .attr("class", "component-d3-bar")
          .attr("x", (d) => (x(d.category) || margin.left) + (xInner(d.comparison || "") || 0))
          .attr("y", (d) => Math.min(y(0), y(d.value)))
          .attr("width", xInner.bandwidth())
          .attr("height", (d) => Math.max(1, Math.abs(y(d.value) - y(0))))
          .style("fill", color)
          .append("title")
          .text((d) => `${d.label}: ${d.display}`);
      }
    } else {
      const x = window.d3
        .scaleLinear()
        .domain([domainMin, domainMax])
        .nice()
        .range([margin.left, width - margin.right]);
      const y = window.d3
        .scaleBand()
        .domain(categories)
        .range([margin.top, height - margin.bottom])
        .padding(0.24);
      const yInner = window.d3
        .scaleBand()
        .domain(comparisons.length ? comparisons : [""])
        .range([0, y.bandwidth()])
        .padding(0.12);

      svg
        .append("g")
        .attr("class", "component-d3-axis")
        .attr("transform", `translate(0,${height - margin.bottom})`)
        .call(window.d3.axisBottom(x).ticks(5).tickSizeOuter(0));
      svg
        .append("g")
        .attr("class", "component-d3-axis")
        .attr("transform", `translate(${x(0)},0)`)
        .call(window.d3.axisLeft(y).tickSizeOuter(0));

      if (layout === "stacked") {
        const byCategory = window.d3.group(data, (d) => d.category);
        categories.forEach((category) => {
          let negative = 0;
          let positive = 0;
          (byCategory.get(category) || []).forEach((datum) => {
            const start = datum.value < 0 ? negative : positive;
            const end = start + datum.value;
            if (datum.value < 0) {
              negative = end;
            } else {
              positive = end;
            }
            const x0 = x(start);
            const x1 = x(end);
            svg
              .append("rect")
              .attr("class", "component-d3-bar")
              .attr("x", Math.min(x0, x1))
              .attr("y", y(category) || margin.top)
              .attr("height", y.bandwidth())
              .attr("width", Math.max(1, Math.abs(x1 - x0)))
              .style("fill", color(datum))
              .append("title")
              .text(`${datum.label}: ${datum.display}`);
          });
        });
      } else {
        svg
          .append("g")
          .selectAll("rect")
          .data(data)
          .join("rect")
          .attr("class", "component-d3-bar")
          .attr("x", (d) => Math.min(x(0), x(d.value)))
          .attr("y", (d) => (y(d.category) || 0) + (yInner(d.comparison || "") || 0))
          .attr("height", yInner.bandwidth())
          .attr("width", (d) => Math.max(1, Math.abs(x(d.value) - x(0))))
          .style("fill", color)
          .append("title")
          .text((d) => `${d.label}: ${d.display}`);
      }
    }

    appendAxisLabels();
  }

  function renderLine(surface, visual) {
    const data = (visual.points || []).map((point) => ({
      label: pointLabel(point),
      value: formatValue(point.value),
      display: point.display_value || String(point.value),
    }));
    const width = 760;
    const height = 300;
    const margin = { top: 18, right: 42, bottom: 62, left: 58 };
    const svg = window.d3
      .select(surface)
      .append("svg")
      .attr("class", "component-d3-svg component-d3-svg--line")
      .attr("viewBox", `0 0 ${width} ${height}`);
    const x = window.d3
      .scalePoint()
      .domain(data.map((d) => d.label))
      .range([margin.left, width - margin.right])
      .padding(0.5);
    const yMin = Math.min(0, window.d3.min(data, (d) => d.value) || 0);
    let yMax = Math.max(0, window.d3.max(data, (d) => d.value) || 0);
    if (yMin === yMax) {
      yMax = yMin + 1;
    }
    const y = window.d3
      .scaleLinear()
      .domain([yMin, yMax])
      .nice()
      .range([height - margin.bottom, margin.top]);
    const line = window.d3
      .line()
      .x((d) => x(d.label) || margin.left)
      .y((d) => y(d.value))
      .curve(visual.line_smoothing === false ? window.d3.curveLinear : window.d3.curveMonotoneX);

    svg
      .append("line")
      .attr("class", "component-d3-zero-line")
      .attr("x1", margin.left)
      .attr("x2", width - margin.right)
      .attr("y1", y(0))
      .attr("y2", y(0));

    svg
      .append("g")
      .attr("class", "component-d3-axis")
      .attr("transform", `translate(0,${height - margin.bottom})`)
      .call(
        window.d3
          .axisBottom(x)
          .tickValues(data.filter((_point, index) => index % Math.max(1, Math.ceil(data.length / 8)) === 0).map((point) => point.label))
          .tickSizeOuter(0)
      )
      .call((axis) =>
        axis
          .selectAll("text")
          .attr("transform", "rotate(-35)")
          .attr("text-anchor", "end")
          .attr("dx", "-0.45em")
          .attr("dy", "0.45em")
      );
    svg
      .append("g")
      .attr("class", "component-d3-axis")
      .attr("transform", `translate(${margin.left},0)`)
      .call(window.d3.axisLeft(y).ticks(4).tickSizeOuter(0));
    svg
      .append("path")
      .datum(data)
      .attr("class", "component-d3-line")
      .attr("d", line);
    svg
      .append("g")
      .selectAll("circle")
      .data(data)
      .join("circle")
      .attr("class", "component-d3-point")
      .attr("cx", (d) => x(d.label) || margin.left)
      .attr("cy", (d) => y(d.value))
      .attr("r", 4)
      .append("title")
      .text((d) => `${d.label}: ${d.display}`);
  }

  function renderSlices(surface, visual) {
    const data = (visual.slices || []).map((slice) => ({
      label: slice.category,
      value: formatValue(slice.value),
      display: slice.display_value || String(slice.value),
      color: slice.color,
    }));
    const width = 540;
    const height = Math.max(250, data.length * 34 + 96);
    const radius = 112;
    const centerX = 128;
    const centerY = height / 2;
    const legendX = 288;
    const legendY = Math.max(36, (height - data.length * 34) / 2);
    const svg = window.d3
      .select(surface)
      .append("svg")
      .attr("class", "component-d3-svg component-d3-svg--slices")
      .attr("viewBox", `0 0 ${width} ${height}`);
    const arc = window.d3
      .arc()
      .innerRadius(visual.component_type === "donut" ? 64 : 0)
      .outerRadius(radius);
    const pie = window.d3
      .pie()
      .sort(null)
      .value((d) => Math.max(0, d.value));
    const fallbackColor = window.d3
      .scaleOrdinal()
      .domain(data.map((d) => d.label))
      .range(["var(--semantic-primary)", "var(--semantic-success)", "var(--semantic-info)", "var(--semantic-warning)", "var(--color-cyan)"]);
    const color = (datum) => datum.color || fallbackColor(datum.label);

    svg
      .append("g")
      .attr("transform", `translate(${centerX},${centerY})`)
      .selectAll("path")
      .data(pie(data))
      .join("path")
      .attr("class", "component-d3-slice")
      .attr("fill", (d) => color(d.data))
      .attr("d", arc)
      .append("title")
      .text((d) => `${d.data.label}: ${d.data.display}`);

    const legend = svg
      .append("g")
      .attr("class", "component-d3-legend")
      .attr("transform", `translate(${legendX},${legendY})`);
    let itemOffset = 0;
    if (visual.legend_title) {
      legend.append("text").attr("class", "component-d3-legend-title").attr("x", 0).attr("y", 0).text(visual.legend_title);
      itemOffset = 28;
    }
    const item = legend
      .selectAll("g")
      .data(data)
      .join("g")
      .attr("transform", (_d, index) => `translate(0,${itemOffset + index * 34})`);
    item.append("rect").attr("width", 14).attr("height", 14).attr("rx", 3).attr("fill", (d) => color(d));
    item.append("text").attr("x", 24).attr("y", 12).text((d) => `${d.label}: ${d.display}`);
  }

  function renderChart(element) {
    const surface = element.querySelector(".component-d3-chart__surface");
    if (!surface) {
      return;
    }
    if (!window.d3) {
      clear(surface);
      const error = document.createElement("p");
      error.className = "component-chart__error";
      error.setAttribute("role", "alert");
      error.textContent = "Chart rendering could not be loaded.";
      surface.appendChild(error);
      return;
    }
    const hasRenderedChart = Boolean(
      surface.querySelector("svg, .component-d3-chart__legend, .component-chart__empty")
    );
    if (element.dataset[renderedKey] === element.dataset.chart && hasRenderedChart) {
      return;
    }
    const visual = parseVisual(element);
    clear(surface);
    if (visual.component_type === "pie" || visual.component_type === "donut") {
      renderSlices(surface, visual);
    } else if (visual.component_type === "line") {
      renderLine(surface, visual);
    } else {
      renderBar(surface, visual);
    }
    element.dataset[renderedKey] = element.dataset.chart || "";
  }

  function renderAll() {
    document.querySelectorAll("[data-chart]").forEach(renderChart);
  }

  function closeFloatingEditorControls(event) {
    const target = event.target;
    if (!(target instanceof Element)) {
      return;
    }
    const currentHelp = target.closest(".component-field-help");
    document.querySelectorAll(".component-field-help[open]").forEach((details) => {
      if (details !== currentHelp) {
        details.removeAttribute("open");
      }
    });
    const currentColorPicker = target.closest(".component-category-labels__color-picker");
    document.querySelectorAll(".component-category-labels__color-picker.is-open").forEach((picker) => {
      if (picker !== currentColorPicker) {
        picker.classList.remove("is-open");
      }
    });
  }

  document.addEventListener("DOMContentLoaded", renderAll);
  document.addEventListener("click", closeFloatingEditorControls);
  window.addEventListener("load", renderAll);
  new MutationObserver(renderAll).observe(document.documentElement, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ["data-chart"],
  });

  window.TessaraCharts = { renderAll };
})();
