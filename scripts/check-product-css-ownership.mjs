import fs from "node:fs";
import postcss from "postcss";

const inputs = [
  ["crates/tessara-component-ui/assets/component.css", ".module-scope--tessara-components"],
  ["crates/tessara-component-ui/assets/component-lifecycle.css", ".module-scope--tessara-components"],
  ["crates/tessara-dashboard-ui/assets/dashboard.css", ".module-scope--tessara-dashboards"],
  ["crates/tessara-dashboard-ui/assets/dashboard-lifecycle.css", ".module-scope--tessara-dashboards"],
  ["crates/tessara-reference-scoped-records/assets/scoped-records.css", ".module-scope--tessara-reference-scoped-records"],
];
const findings = [];
for (const [file, namespace] of inputs) {
  const css = postcss.parse(fs.readFileSync(file, "utf8"), { from: file });
  css.walkDecls(decl => {
    if (/^--(?:color|semantic|font|radius|shadow)/.test(decl.prop)) {
      findings.push({ code: "product_design_token", path: file, message: `product token declaration ${decl.prop}` });
    }
  });
  css.walkRules(rule => {
    if (rule.parent?.type === "atrule" && /keyframes$/i.test(rule.parent.name)) return;
    for (const selector of rule.selectors) {
      const value = selector.trim();
      const rooted = value === namespace || [" ", ":", "[", ".", "#"].some(separator => value.startsWith(namespace + separator));
      if (!rooted) findings.push({ code: "unnamespaced_product_selector", path: file, message: value });
    }
  });
}
process.stdout.write(JSON.stringify({ findings, passed: findings.length === 0 }));
if (findings.length) process.exitCode = 1;
