(function () {
  "use strict";

  var API_CONFIG = "/api/config";
  var API_GRAPH = "/api/config/graph";
  var SVG_NS = "http://www.w3.org/2000/svg";
  var FOCAL_ROUTE_LIMIT = 14;
  var PROVIDERS = ["bedrock", "codex", "google"];
  var SOURCE_FORMATS = ["", "responses", "chat_completions", "anthropic_messages", "google_generate_content", "openai_images"];
  var TARGET_FORMATS = ["", "responses", "anthropic_messages", "google_generate_content", "openai_images"];
  var providerStyles = [
    { className: "provider-palette-0", patterned: false },
    { className: "provider-palette-1", patterned: true },
    { className: "provider-palette-2", patterned: true },
    { className: "provider-palette-3", patterned: true },
    { className: "provider-palette-4", patterned: true },
    { className: "provider-palette-5", patterned: true }
  ];

  var state = {
    currentConfig: null,
    draftConfig: null,
    currentGraph: null,
    selectedRouteId: null,
    selectedEdgeId: null,
    selectedDraftRouteIndex: 0,
    providerStyleByName: new Map(),
    isBusy: false,
    draftNeedsValidation: false,
    lastValidatedDraft: "",
    saveBlockedReason: ""
  };

  var elements = {};

  document.addEventListener("DOMContentLoaded", init);

  function init() {
    elements.root = document.querySelector("[data-config-app]");
    if (!elements.root) {
      return;
    }

    elements.statusDot = document.querySelector("[data-status-dot]");
    elements.statusText = document.querySelector("[data-status-text]");
    elements.emptyState = document.querySelector("[data-empty-state]");
    elements.errorState = document.querySelector("[data-error-state]");
    elements.legend = document.querySelector("[data-provider-legend]");
    elements.map = document.querySelector("[data-route-map]");
    elements.routesBody = document.querySelector("[data-routes-body]");
    elements.routeCards = document.querySelector("[data-route-cards]");
    elements.inspector = document.querySelector("[data-inspector]");
    elements.tableSummary = document.querySelector("[data-table-summary]");
    elements.diagnostics = document.querySelector("[data-diagnostics]");
    elements.rawDrawer = document.querySelector("[data-raw-drawer]");
    elements.currentEditor = document.querySelector("[data-current-editor]");
    elements.draftEditor = document.querySelector("[data-draft-editor]");
    elements.editorStatus = document.querySelector("[data-editor-status]");
    elements.validateDraft = document.querySelector("[data-validate-draft]");
    elements.saveConfig = document.querySelector("[data-save-config]");
    elements.revertConfig = document.querySelector("[data-revert-config]");
    elements.themeToggle = document.querySelector("[data-theme-toggle]");

    initTheme();
    bindTypedEditor();

    elements.validateDraft.addEventListener("click", function () {
      validateDraft();
    });
    elements.saveConfig.addEventListener("click", saveDraft);
    elements.revertConfig.addEventListener("click", loadConfig);
    elements.draftEditor.addEventListener("input", handleRawDraftInput);
    elements.draftEditor.addEventListener("keydown", handleEditorKeydown);
    elements.routesBody.addEventListener("click", handleRouteClick);
    elements.routesBody.addEventListener("keydown", handleRouteListKeydown);
    addHandler(elements.themeToggle, "click", toggleTheme);
    elements.routeCards.addEventListener("click", handleRouteClick);
    elements.routeCards.addEventListener("keydown", handleRouteListKeydown);
    document.addEventListener("keydown", handleGlobalKeydown);

    loadConfig();
  }

  function bindTypedEditor() {
    elements.routeEditorForm = document.querySelector("[data-route-editor-form]");
    elements.typedRouteList = document.querySelector("[data-draft-route-list]");
    elements.sourceModelInput = document.querySelector("[data-route-source-model]");
    elements.sourceFormatInput = document.querySelector("[data-route-runtime-format]");
    elements.targetProviderInput = document.querySelector("[data-route-target-provider]");
    elements.targetModelInput = document.querySelector("[data-route-target-model]");
    elements.targetFormatInput = document.querySelector("[data-route-target-format]");
    elements.enabledInput = document.querySelector("[data-route-enabled]");
    elements.newRouteButton = document.querySelector("[data-route-create]");
    elements.updateRouteButton = document.querySelector("[data-route-update]");
    elements.previewRouteButton = document.querySelector("[data-route-preview]");
    elements.routeValidateButton = document.querySelector("[data-route-validate]");
    elements.routeSaveButton = document.querySelector("[data-route-save]");
    elements.moveUpButton = document.querySelector("[data-route-move-up]");
    elements.moveDownButton = document.querySelector("[data-route-move-down]");
    elements.enableButton = document.querySelector("[data-route-enable]");
    elements.disableButton = document.querySelector("[data-route-disable]");
    elements.routeEditorStatus = document.querySelector("[data-route-editor-status]");

    addHandler(elements.routeEditorForm, "submit", function (event) {
      event.preventDefault();
      validateDraft();
    });

    [
      elements.sourceModelInput,
      elements.sourceFormatInput,
      elements.targetProviderInput,
      elements.targetModelInput,
      elements.targetFormatInput,
      elements.enabledInput
    ].forEach(function (input) {
      addHandler(input, "input", applyTypedFormToDraft);
      addHandler(input, "change", applyTypedFormToDraft);
    });

    addHandler(elements.newRouteButton, "click", createDraftRoute);
    addHandler(elements.updateRouteButton, "click", handleManualRouteUpdate);
    addHandler(elements.previewRouteButton, "click", validateDraft);
    addHandler(elements.routeValidateButton, "click", validateDraft);
    addHandler(elements.routeSaveButton, "click", saveDraft);
    addHandler(elements.moveUpButton, "click", function () {
      moveDraftRoute(-1);
    });
    addHandler(elements.moveDownButton, "click", function () {
      moveDraftRoute(1);
    });
    addHandler(elements.enableButton, "click", function () {
      setDraftRouteEnabled(true);
    });
    addHandler(elements.disableButton, "click", function () {
      setDraftRouteEnabled(false);
    });
    addHandler(elements.typedRouteList, "click", handleTypedRouteListClick);
    addHandler(elements.typedRouteList, "keydown", handleTypedRouteListKeydown);
  }

  function addHandler(element, eventName, handler) {
    if (element) {
      element.addEventListener(eventName, handler);
    }
  }

  function initTheme() {
    var storedTheme = safeStorageGet("ump-config-theme");
    var theme = storedTheme === "dark" || storedTheme === "light" ? storedTheme : preferredTheme();
    applyTheme(theme, false);
  }

  function toggleTheme() {
    var current = document.documentElement.dataset.theme || preferredTheme();
    applyTheme(current === "dark" ? "light" : "dark", true);
  }

  function applyTheme(theme, persist) {
    document.documentElement.dataset.theme = theme;
    if (elements.themeToggle) {
      var isDark = theme === "dark";
      elements.themeToggle.textContent = isDark ? "Light" : "Dark";
      elements.themeToggle.setAttribute("aria-pressed", String(isDark));
    }
    if (persist) {
      safeStorageSet("ump-config-theme", theme);
    }
  }

  function preferredTheme() {
    if (window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches) {
      return "dark";
    }
    return "light";
  }

  function safeStorageGet(key) {
    try {
      return window.localStorage.getItem(key);
    } catch (error) {
      return null;
    }
  }

  function safeStorageSet(key, value) {
    try {
      window.localStorage.setItem(key, value);
    } catch (error) {
      return;
    }
  }

  function labeledInput(parent, labelText, type) {
    var label = document.createElement("label");
    var labelSpan = document.createElement("span");
    var input = document.createElement("input");
    label.className = "editor-field";
    labelSpan.textContent = labelText;
    input.type = type;
    input.autocomplete = "off";
    label.appendChild(labelSpan);
    label.appendChild(input);
    parent.appendChild(label);
    return input;
  }

  function labeledSelect(parent, labelText, values) {
    var label = document.createElement("label");
    var labelSpan = document.createElement("span");
    var select = document.createElement("select");
    label.className = "editor-field";
    labelSpan.textContent = labelText;
    values.forEach(function (value) {
      var option = document.createElement("option");
      option.value = value;
      option.textContent = value || "provider default";
      select.appendChild(option);
    });
    label.appendChild(labelSpan);
    label.appendChild(select);
    parent.appendChild(label);
    return select;
  }

  function labeledCheckbox(parent, labelText) {
    var label = document.createElement("label");
    var labelSpan = document.createElement("span");
    var input = document.createElement("input");
    label.className = "editor-field";
    labelSpan.textContent = labelText;
    input.type = "checkbox";
    label.appendChild(labelSpan);
    label.appendChild(input);
    parent.appendChild(label);
    return input;
  }

  function actionButton(label, extraClass, handler) {
    var button = document.createElement("button");
    button.type = "button";
    button.className = "button" + (extraClass ? " " + extraClass : "");
    button.textContent = label;
    button.addEventListener("click", handler);
    return button;
  }

  function loadConfig() {
    setBusy(true, "Loading route projection…");
    clearError();

    return Promise.all([fetchJson(API_CONFIG), fetchJson(API_GRAPH)])
      .then(function (responses) {
        state.currentConfig = responses[0] || { routes: [] };
        state.draftConfig = cloneConfig(state.currentConfig);
        state.currentGraph = responses[1] || {};
        state.selectedRouteId = null;
        state.selectedEdgeId = edgeIdForRoute(state.currentGraph, state.selectedRouteId);
        state.selectedDraftRouteIndex = selectedIndexFromRouteId(state.currentGraph, state.selectedRouteId);
        state.draftNeedsValidation = false;
        state.lastValidatedDraft = stableDraftText();
        state.saveBlockedReason = blockingReason(state.currentGraph);
        syncEditors();
        render();
        setReady("Route projection loaded.");
        setEditorStatus(draftStatusText(state.currentGraph));
      })
      .catch(function (error) {
        showError(error);
      })
      .finally(function () {
        setBusy(false);
      });
  }

  function validateDraft(options) {
    var draft = parseDraft();
    if (!draft.ok) {
      setEditorStatus(draft.message);
      showError(new Error(draft.message));
      updateActionButtons();
      return Promise.resolve(null);
    }

    adoptDraftConfig(draft.value);
    setBusy(true, "Validating draft…");
    clearError();

    return fetchJson(API_GRAPH, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(draft.value)
    })
      .then(function (graph) {
        state.currentGraph = graph || {};
        state.selectedRouteId = null;
        state.selectedEdgeId = edgeIdForRoute(graph, state.selectedRouteId);
        state.selectedDraftRouteIndex = selectedIndexFromRouteId(graph, state.selectedRouteId);
        state.draftNeedsValidation = false;
        state.lastValidatedDraft = stableDraftText();
        state.saveBlockedReason = blockingReason(graph);
        render();
        if (state.saveBlockedReason) {
          showError(new Error(state.saveBlockedReason));
          setEditorStatus(state.saveBlockedReason);
        } else {
          setReady("Draft projection rendered. Not saved.");
          setEditorStatus(draftStatusText(graph) || "Draft is valid. Generated JSON is ready to save.");
        }
        if (options && options.saveAfter && !state.saveBlockedReason) {
          return putDraft(draft.value);
        }
        return graph;
      })
      .catch(function (error) {
        showError(error);
        setEditorStatus(error.message);
        return null;
      })
      .finally(function () {
        setBusy(false);
      });
  }

  function saveDraft() {
    var draft = parseDraft();
    if (!draft.ok) {
      setEditorStatus(draft.message);
      showError(new Error(draft.message));
      updateActionButtons();
      return;
    }

    adoptDraftConfig(draft.value);

    if (state.draftNeedsValidation || stableDraftText() !== state.lastValidatedDraft) {
      validateDraft({ saveAfter: true });
      return;
    }

    state.saveBlockedReason = blockingReason(state.currentGraph);
    if (state.saveBlockedReason) {
      showError(new Error(state.saveBlockedReason));
      setEditorStatus(state.saveBlockedReason);
      updateActionButtons();
      return;
    }

    putDraft(draft.value);
  }

  function putDraft(value) {
    setBusy(true, "Saving draft…");
    clearError();

    return fetchJson(API_CONFIG, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(value)
    })
      .then(function () {
        setEditorStatus("Saved. Reloading persisted projection…");
        return loadConfig();
      })
      .catch(function (error) {
        showError(error);
        setEditorStatus(error.message);
      })
      .finally(function () {
        setBusy(false);
      });
  }

  function handleRawDraftInput() {
    state.draftNeedsValidation = true;
    state.saveBlockedReason = "Validate the draft before saving.";
    var parsed = parseDraft();
    if (parsed.ok) {
      adoptDraftConfig(parsed.value, { keepPreview: true });
      renderTypedEditor();
      setEditorStatus("Raw draft changed. Validate to refresh the projection.");
    } else {
      setEditorStatus(parsed.message);
    }
    updateActionButtons();
  }

  function handleEditorKeydown(event) {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      validateDraft();
    }

    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
      event.preventDefault();
      saveDraft();
    }
  }

  function handleGlobalKeydown(event) {
    if (event.defaultPrevented) {
      return;
    }

    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "j") {
      event.preventDefault();
      openRawDrawer({ focus: "draft" });
      return;
    }

    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "e") {
      event.preventDefault();
      focusRouteEditor();
      return;
    }

    if (event.altKey && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
      event.preventDefault();
      selectAdjacentRoute(event.key === "ArrowDown" ? 1 : -1);
    }
  }

  function handleRouteClick(event) {
    var trigger = event.target.closest("[data-route-select]");
    if (trigger) {
      selectRoute(trigger.dataset.routeSelect);
    }
  }

  function handleRouteListKeydown(event) {
    var trigger = event.target.closest("[data-route-select]");
    if (!trigger) {
      return;
    }

    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      selectRoute(trigger.dataset.routeSelect);
      return;
    }

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      selectAdjacentRoute(event.key === "ArrowDown" ? 1 : -1);
    }
  }

  function handleTypedRouteListClick(event) {
    var trigger = event.target.closest("[data-draft-route-index]");
    if (trigger) {
      selectDraftRoute(Number(trigger.dataset.draftRouteIndex));
    }
  }

  function handleTypedRouteListKeydown(event) {
    var trigger = event.target.closest("[data-draft-route-index]");
    if (!trigger) {
      return;
    }

    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      selectDraftRoute(Number(trigger.dataset.draftRouteIndex));
    }

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      selectDraftRoute(Number(trigger.dataset.draftRouteIndex) + (event.key === "ArrowDown" ? 1 : -1));
    }
  }

  function openRawDrawer(options) {
    if (!elements.rawDrawer) {
      return;
    }
    elements.rawDrawer.open = true;

    if (!prefersReducedMotion() && elements.rawDrawer.scrollIntoView) {
      elements.rawDrawer.scrollIntoView({ block: "start", behavior: "smooth" });
    } else if (elements.rawDrawer.scrollIntoView) {
      elements.rawDrawer.scrollIntoView({ block: "start" });
    }

    if (options && options.focus === "typed" && elements.sourceModelInput) {
      focusRouteEditor({ preventScroll: true });
      return;
    }

    if (elements.draftEditor) {
      elements.draftEditor.focus({ preventScroll: true });
    }
  }

  function focusRouteEditor(options) {
    var panel = document.querySelector("[data-route-editor-region]");
    if (panel && panel.tagName && panel.tagName.toLowerCase() === "details") {
      panel.open = true;
    }
    if (panel && panel.scrollIntoView && !(options && options.preventScroll)) {
      panel.scrollIntoView({ block: "start", behavior: prefersReducedMotion() ? "auto" : "smooth" });
    }
    if (elements.sourceModelInput) {
      elements.sourceModelInput.focus({ preventScroll: !!(options && options.preventScroll) });
    }
  }

  function fetchJson(url, options) {
    return fetch(url, Object.assign({ headers: { "Accept": "application/json" } }, options || {}))
      .then(function (response) {
        return response.text().then(function (text) {
          var body = text ? parseJsonText(text, url) : null;
          if (!response.ok) {
            throw new Error(errorMessageFromBody(body, response));
          }
          return body;
        });
      });
  }

  function parseJsonText(text, label) {
    try {
      return JSON.parse(text);
    } catch (error) {
      throw new Error("Invalid JSON from " + label + ".");
    }
  }

  function errorMessageFromBody(body, response) {
    if (body && body.error && body.error.message) {
      return body.error.message;
    }
    if (body && body.message) {
      return body.message;
    }
    return "Request failed with HTTP " + response.status + ".";
  }

  function parseDraft() {
    try {
      return { ok: true, value: normalizeConfig(JSON.parse(elements.draftEditor.value)) };
    } catch (error) {
      return { ok: false, message: "Draft JSON is invalid: " + error.message };
    }
  }

  function adoptDraftConfig(config, options) {
    state.draftConfig = normalizeConfig(config);
    clampSelectedDraftIndex();
    if (!options || !options.keepPreview) {
      syncDraftPreview();
    }
  }

  function syncEditors() {
    elements.currentEditor.value = formatJson(state.currentConfig || { routes: [] });
    syncDraftPreview();
    renderTypedEditor();
    updateActionButtons();
  }

  function syncDraftPreview() {
    elements.draftEditor.value = formatJson(state.draftConfig || { routes: [] });
  }

  function render() {
    var graph = state.currentGraph || {};
    var routes = effectiveRoutes(graph);
    var cardRoutes = focalRoutes(graph);
    assignProviderStyles(graph, routes.concat(cardRoutes));
    renderEmptyState(graph, routes);
    renderLegend(graph, routes.concat(cardRoutes));
    renderTable(routes);
    renderCards(graph, cardRoutes);
    renderMap(graph, cardRoutes);
    renderInspector(selectedRoute(routes.concat(cardRoutes)));
    renderDiagnostics(graph);
    renderTypedEditor();
    renderDraftStatus(graph);
    elements.tableSummary.textContent = routeSummaryText(routes);
    updateActionButtons();
  }

  function effectiveRoutes(graph) {
    if (Array.isArray(graph.effective_routes)) {
      return graph.effective_routes;
    }
    if (Array.isArray(graph.routes)) {
      return graph.routes;
    }
    return [];
  }

  function configRoutes(graph) {
    return Array.isArray(graph.config_routes) ? graph.config_routes : [];
  }

  function graphNodes(graph) {
    return Array.isArray(graph.nodes) ? graph.nodes : [];
  }

  function graphEdges(graph) {
    return Array.isArray(graph.edges) ? graph.edges : [];
  }

  function graphGroups(graph) {
    return Array.isArray(graph.groups) ? graph.groups : [];
  }

  function routeCards(graph) {
    if (!Array.isArray(graph.route_cards)) {
      return [];
    }
    return graph.route_cards.map(function (card, index) {
      return normalizeRouteCard(card, index);
    });
  }

  function diagnostics(graph) {
    if (Array.isArray(graph.diagnostics_v2)) {
      return graph.diagnostics_v2;
    }
    return [].concat(
      Array.isArray(graph.diagnostics) ? graph.diagnostics : [],
      Array.isArray(graph.validation_issues) ? graph.validation_issues : []
    );
  }

  function focalRoutes(graph) {
    var cards = routeCards(graph);
    if (cards.length) {
      return firstPaintRoutes(cards);
    }

    var hotRoutes = effectiveRoutes(graph).filter(function (route) {
      return isHotRoute(route);
    });
    if (hotRoutes.length) {
      return firstPaintRoutes(hotRoutes);
    }

    var configuredRoutes = configRoutes(graph).filter(function (route) {
      return route.enabled !== false;
    });
    if (configuredRoutes.length) {
      return firstPaintRoutes(configuredRoutes);
    }

    return firstPaintRoutes(effectiveRoutes(graph));
  }

  function firstPaintRoutes(routes) {
    var selected = [];
    var seen = new Set();

    routes.forEach(function (route, index) {
      if (isPriorityRoute(route)) {
        addFocalRoute(selected, seen, route, index);
      }
    });

    providerNames({}, routes).forEach(function (provider) {
      var index = routes.findIndex(function (route) {
        return targetProviderName(route) === provider && !seen.has(routeIdentity(route, routes.indexOf(route)));
      });
      if (index >= 0) {
        addFocalRoute(selected, seen, routes[index], index);
      }
    });

    routes.forEach(function (route, index) {
      if (selected.length < FOCAL_ROUTE_LIMIT) {
        addFocalRoute(selected, seen, route, index);
      }
    });

    return selected.length ? selected : routes.slice(0, FOCAL_ROUTE_LIMIT);
  }

  function addFocalRoute(selected, seen, route, index) {
    if (selected.length >= FOCAL_ROUTE_LIMIT) {
      return;
    }
    var id = routeIdentity(route, index);
    if (!seen.has(id)) {
      seen.add(id);
      selected.push(route);
    }
  }

  function isPriorityRoute(route) {
    var stateText = stringValue(route.state || route.status || route.kind, "").toLowerCase();
    return isHotRoute(route) || route.enabled === false || stateText === "invalid" || stateText === "disabled" || stateText === "shadowed";
  }

  function normalizeRouteCard(card, index) {
    card = card || {};
    var route = card && typeof card.route === "object" ? cloneConfig(card.route) : cloneConfig(card || {});
    route.__card_title = stringValue(card.title || card.label || route.title, "");
    route.__card_subtitle = stringValue(card.subtitle || card.description || route.subtitle, "");
    route.__card_group = stringValue(card.group || card.group_id || card.group_name, "");
    route.__card_index = index;
    if (!route.route_id && card.route_id) {
      route.route_id = card.route_id;
    }
    if (!route.id && card.id) {
      route.id = card.id;
    }
    return route;
  }

  function renderEmptyState(graph, routes) {
    var hotRoutes = configRoutes(graph).filter(function (route) {
      return isHotRoute(route);
    });
    elements.emptyState.hidden = !(hotRoutes.length === 0 && routes.length > 0);
  }

  function renderLegend(graph, routes) {
    clearElement(elements.legend);
    providerNames(graph, routes).forEach(function (provider) {
      var palette = providerStyle(provider);
      var item = document.createElement("span");
      item.className = "legend__item";

      var swatch = document.createElement("span");
      swatch.className = "legend__swatch";
      swatch.classList.add(palette.className);
      if (palette.patterned) {
        swatch.classList.add("provider-pattern");
      }

      item.appendChild(swatch);
      item.appendChild(document.createTextNode(provider));
      elements.legend.appendChild(item);
    });
  }

  function renderTable(routes) {
    clearElement(elements.routesBody);

    if (!routes.length) {
      var emptyRow = document.createElement("tr");
      var emptyCell = document.createElement("td");
      emptyCell.colSpan = 4;
      emptyCell.appendChild(paragraph("muted", "No effective routes reported by the server."));
      emptyRow.appendChild(emptyCell);
      elements.routesBody.appendChild(emptyRow);
      return;
    }

    groupRoutesForLedger(routes).forEach(function (providerGroup) {
      renderProviderLedgerRow(providerGroup);
      providerGroup.endpoints.forEach(function (endpointGroup) {
        renderEndpointLedgerRow(providerGroup, endpointGroup);
      });
    });
  }

  function renderProviderLedgerRow(providerGroup) {
    var row = document.createElement("tr");
    row.className = "provider-row";
    applyProviderStyle(row, providerGroup.provider);

    var cell = document.createElement("td");
    cell.colSpan = 4;

    var heading = document.createElement("div");
    heading.className = "provider-heading";
    var swatch = document.createElement("span");
    swatch.className = "provider-heading__swatch";
    var name = strong(providerDisplayName(providerGroup.provider));
    var meta = span(
      "subtle",
      providerGroup.endpoints.length + " " + pluralize(providerGroup.endpoints.length, "endpoint") +
      " · " + providerGroup.routeCount + " " + pluralize(providerGroup.routeCount, "route")
    );
    heading.appendChild(swatch);
    heading.appendChild(name);
    heading.appendChild(meta);
    cell.appendChild(heading);
    row.appendChild(cell);
    elements.routesBody.appendChild(row);
  }

  function renderEndpointLedgerRow(providerGroup, endpointGroup) {
    var firstEntry = endpointGroup.entries[0];
    var row = document.createElement("tr");
    row.className = "endpoint-row";
    row.setAttribute("aria-current", String(endpointContainsSelectedRoute(endpointGroup)));
    applyProviderStyle(row, providerGroup.provider);

    var railCell = document.createElement("td");
    railCell.appendChild(span("endpoint-rail", "↳"));

    var endpointCell = document.createElement("td");
    var endpointStack = document.createElement("div");
    endpointStack.className = "endpoint-stack";
    endpointStack.appendChild(strong(endpointLabel(endpointGroup.format)));
    endpointStack.appendChild(span("endpoint-meta", endpointGroup.format === "*" ? "Matches any runtime format" : formatDisplayName(endpointGroup.format)));
    endpointCell.appendChild(endpointStack);

    var modelsCell = document.createElement("td");
    if (endpointGroup.entries.length === 1) {
      modelsCell.appendChild(routeModelButton(firstEntry.route, firstEntry.index));
    } else {
      modelsCell.appendChild(modelDisclosure(endpointGroup));
    }

    var stateCell = document.createElement("td");
    var chipRow = document.createElement("div");
    chipRow.className = "endpoint-chip-row";
    endpointChips(endpointGroup).forEach(function (routeChip) {
      chipRow.appendChild(chip(routeChip.label, routeChip.kind));
    });
    stateCell.appendChild(chipRow);

    row.appendChild(railCell);
    row.appendChild(endpointCell);
    row.appendChild(modelsCell);
    row.appendChild(stateCell);
    elements.routesBody.appendChild(row);
  }

  function modelDisclosure(endpointGroup) {
    var details = document.createElement("details");
    details.className = "model-disclosure";
    if (endpointContainsSelectedRoute(endpointGroup)) {
      details.open = true;
    }

    var summary = document.createElement("summary");
    var label = document.createElement("span");
    label.className = "target-stack";
    label.appendChild(strong(primaryTargetLabel(endpointGroup.entries)));
    label.appendChild(span("subtle", sourceModelSummary(endpointGroup.entries)));

    var count = chip(sourceModelCount(endpointGroup.entries) + " " + pluralize(sourceModelCount(endpointGroup.entries), "model"), "count");
    count.classList.add("model-count");
    count.tabIndex = 0;
    count.dataset.modelPreview = modelPreview(endpointGroup.entries);

    summary.appendChild(label);
    summary.appendChild(count);
    details.appendChild(summary);

    var list = document.createElement("div");
    list.className = "model-list";
    endpointGroup.entries.forEach(function (entry) {
      list.appendChild(routeModelButton(entry.route, entry.index));
    });
    details.appendChild(list);
    return details;
  }

  function routeModelButton(route, index) {
    var routeId = routeIdentity(route, index);
    var button = document.createElement("button");
    button.type = "button";
    button.className = "route-model-button";
    if (isHotRoute(route)) {
      button.classList.add("route-model-button--adapter");
    }
    button.dataset.routeSelect = routeId;
    button.setAttribute("aria-current", String(routeId === state.selectedRouteId));
    button.appendChild(strong(sourceModelName(route) + " → " + targetModelName(route)));
    button.appendChild(span("subtle", targetFormat(route) + " · " + routeLabel(route, index)));
    return button;
  }

  function groupRoutesForLedger(routes) {
    var providerGroups = new Map();
    routes.forEach(function (route, index) {
      var provider = targetProviderName(route);
      var format = runtimeFormat(route);
      if (!providerGroups.has(provider)) {
        providerGroups.set(provider, { provider: provider, routeCount: 0, endpointsByFormat: new Map(), endpoints: [] });
      }
      var providerGroup = providerGroups.get(provider);
      providerGroup.routeCount += 1;
      if (!providerGroup.endpointsByFormat.has(format)) {
        var endpointGroup = { format: format, entries: [] };
        providerGroup.endpointsByFormat.set(format, endpointGroup);
        providerGroup.endpoints.push(endpointGroup);
      }
      providerGroup.endpointsByFormat.get(format).entries.push({ route: route, index: index });
    });

    var groups = Array.from(providerGroups.values());
    groups.forEach(function (group) {
      group.endpoints.forEach(function (endpointGroup) {
        endpointGroup.entries.sort(function (a, b) {
          return Number(isHotRoute(a.route)) - Number(isHotRoute(b.route)) ||
            sourceModelName(a.route).localeCompare(sourceModelName(b.route));
        });
      });
      group.endpoints.sort(function (a, b) {
        return endpointOrder(a.format) - endpointOrder(b.format) || a.format.localeCompare(b.format);
      });
      delete group.endpointsByFormat;
    });
    groups.sort(function (a, b) {
      return providerOrder(a.provider) - providerOrder(b.provider) || a.provider.localeCompare(b.provider);
    });
    return groups;
  }

  function endpointChips(endpointGroup) {
    var chips = [{ label: endpointGroup.entries.length + " " + pluralize(endpointGroup.entries.length, "route"), kind: "count" }];
    var flags = [];
    endpointGroup.entries.forEach(function (entry) {
      routeChips(entry.route).forEach(function (routeChip) {
        if (
          routeChip.kind !== "format" &&
          routeChip.kind !== "catalog" &&
          routeChip.kind !== "hot" &&
          routeChip.kind !== "override" &&
          routeChip.kind !== "count" &&
          !flags.some(function (flag) { return flag.kind === routeChip.kind; })
        ) {
          flags.push(routeChip);
        }
      });
    });
    return chips.concat(flags.slice(0, 3));
  }

  function endpointContainsSelectedRoute(endpointGroup) {
    return endpointGroup.entries.some(function (entry) {
      return routeIdentity(entry.route, entry.index) === state.selectedRouteId;
    });
  }

  function routeSummaryText(routes) {
    var groups = groupRoutesForLedger(routes);
    var endpointCount = groups.reduce(function (total, group) {
      return total + group.endpoints.length;
    }, 0);
    return groups.length + " " + pluralize(groups.length, "provider") +
      " · " + endpointCount + " " + pluralize(endpointCount, "endpoint") +
      " · " + routes.length + " " + pluralize(routes.length, "route");
  }

  function providerDisplayName(provider) {
    var labels = {
      bedrock: "Bedrock Mantle",
      codex: "Codex OAuth",
      google: "Google direct"
    };
    return labels[provider] || provider;
  }

  function providerOrder(provider) {
    var index = PROVIDERS.indexOf(provider);
    return index === -1 ? PROVIDERS.length : index;
  }

  function endpointOrder(format) {
    var index = SOURCE_FORMATS.indexOf(format === "*" ? "" : format);
    return index === -1 ? SOURCE_FORMATS.length : index;
  }

  function endpointLabel(format) {
    var labels = {
      "*": "Any endpoint",
      "": "Any endpoint",
      responses: "POST /v1/responses",
      chat_completions: "POST /v1/chat/completions",
      anthropic_messages: "POST /v1/messages",
      google_generate_content: "POST /v1beta/models/{model}:generateContent",
      openai_images: "POST /v1/images/generations"
    };
    return labels[format] || format;
  }

  function formatDisplayName(format) {
    var labels = {
      "*": "Any runtime",
      "": "Any runtime",
      responses: "OpenAI Responses",
      chat_completions: "OpenAI Chat Completions",
      anthropic_messages: "Anthropic Messages",
      google_generate_content: "Google GenerateContent",
      openai_images: "OpenAI Images"
    };
    return labels[format] || format;
  }

  function primaryTargetLabel(entries) {
    var targets = uniqueValues(entries.map(function (entry) {
      return targetModelName(entry.route);
    }));
    if (targets.length === 1) {
      return targets[0];
    }
    return targets[0] + " +" + (targets.length - 1) + " " + pluralize(targets.length - 1, "target");
  }

  function sourceModelSummary(entries) {
    return sourceModelCount(entries) + " source " + pluralize(sourceModelCount(entries), "model") + " mapped to this endpoint";
  }

  function sourceModelCount(entries) {
    return uniqueValues(entries.map(function (entry) {
      return sourceModelName(entry.route);
    })).length;
  }

  function modelPreview(entries) {
    var models = uniqueValues(entries.map(function (entry) {
      return sourceModelName(entry.route);
    }));
    var preview = models.slice(0, 6).join(", ");
    if (models.length > 6) {
      preview += ", +" + (models.length - 6) + " more";
    }
    return preview;
  }

  function uniqueValues(values) {
    return Array.from(new Set(values.filter(Boolean)));
  }

  function renderCards(graph, routes) {
    clearElement(elements.routeCards);

    var groups = groupRoutesForCards(graph, routes);
    groups.forEach(function (group) {
      if (group.label && groups.length > 1) {
        var heading = document.createElement("h3");
        heading.textContent = group.label;
        elements.routeCards.appendChild(heading);
      }

      group.routes.forEach(function (route, index) {
        elements.routeCards.appendChild(routeCardButton(route, group.offset + index));
      });
    });
  }

  function groupRoutesForCards(graph, routes) {
    var groups = graphGroups(graph);
    if (!groups.length) {
      return [{ label: "", routes: routes, offset: 0 }];
    }

    var groupedRoutes = [];
    var consumedRouteIds = new Set();
    groups.forEach(function (group) {
      var groupCards = [];
      if (Array.isArray(group.route_cards)) {
        groupCards = group.route_cards.map(function (card, index) {
          var normalized = normalizeRouteCard(card, index);
          normalized.__card_group = stringValue(group.title || group.label || group.name || group.id, normalized.__card_group);
          return normalized;
        });
      } else if (Array.isArray(group.routes)) {
        groupCards = group.routes;
      } else {
        groupCards = routes.filter(function (route) {
          return route.__card_group && route.__card_group === stringValue(group.id || group.name || group.label, "");
        });
      }

      groupCards.forEach(function (route) {
        consumedRouteIds.add(routeIdentity(route, groupCards.indexOf(route)));
      });

      if (groupCards.length) {
        groupedRoutes.push({
          label: stringValue(group.title || group.label || group.name || group.id, "Route group"),
          routes: groupCards,
          offset: groupedRoutes.reduce(function (total, entry) { return total + entry.routes.length; }, 0)
        });
      }
    });

    var ungrouped = routes.filter(function (route, index) {
      return !consumedRouteIds.has(routeIdentity(route, index));
    });
    if (ungrouped.length) {
      groupedRoutes.push({ label: "Other focus routes", routes: ungrouped, offset: routes.length - ungrouped.length });
    }

    return groupedRoutes.length ? groupedRoutes : [{ label: "", routes: routes, offset: 0 }];
  }

  function routeCardButton(route, index) {
    var routeId = routeIdentity(route, index);
    var card = document.createElement("button");
    card.type = "button";
    card.className = "route-card";
    card.dataset.routeSelect = routeId;
    card.setAttribute("aria-current", String(routeId === state.selectedRouteId));
    applyProviderStyle(card, targetProviderName(route));

    card.appendChild(strong(route.__card_title || sourceModelName(route)));
    card.appendChild(span("subtle", route.__card_subtitle || runtimeFormat(route) + " → " + targetProviderName(route) + " / " + targetModelName(route)));
    var chipRow = document.createElement("span");
    chipRow.className = "chip-row";
    routeChips(route).forEach(function (routeChip) {
      chipRow.appendChild(chip(routeChip.label, routeChip.kind));
    });
    card.appendChild(chipRow);
    return card;
  }

  function renderMap(graph, routes) {
    clearElement(elements.map);
    addArrowMarker(elements.map);

    var parts = visibleMapParts(graph, routes);
    var layout = layoutNodes(parts.nodes, graphGroups(graph));
    var width = Math.max(760, layout.width);
    var height = Math.max(360, layout.height);

    elements.map.setAttribute("viewBox", "0 0 " + width + " " + height);
    elements.map.setAttribute("width", String(width));
    elements.map.setAttribute("height", String(height));

    layout.groups.forEach(function (group) {
      renderGroupLabel(group);
    });

    parts.edges.forEach(function (edge, index) {
      renderEdge(edge, index, layout.positions);
    });

    parts.nodes.forEach(function (node, index) {
      renderNode(node, index, layout.positions);
    });
  }

  function visibleMapParts(graph, routes) {
    var groups = graphGroups(graph);
    var groupNodes = [];
    var groupEdges = [];

    groups.forEach(function (group) {
      if (Array.isArray(group.nodes)) {
        groupNodes = groupNodes.concat(group.nodes);
      }
      if (Array.isArray(group.edges)) {
        groupEdges = groupEdges.concat(group.edges);
      }
    });

    if (groupNodes.length || groupEdges.length) {
      return compactMapParts(groupNodes, groupEdges, routes);
    }

    var routeIds = new Set(routes.map(function (route, index) {
      return routeIdentity(route, index);
    }));
    var edges = graphEdges(graph).filter(function (edge) {
      return routeIds.has(edgeRouteId(edge));
    });
    var nodeIds = new Set();
    edges.forEach(function (edge) {
      nodeIds.add(edgeFrom(edge));
      nodeIds.add(edgeTo(edge));
    });
    var nodes = graphNodes(graph).filter(function (node, index) {
      return nodeIds.has(nodeIdentity(node, index));
    });

    if (!nodes.length || !edges.length) {
      nodes = nodesFromRoutes(routes);
      edges = edgesFromRoutes(routes);
    }

    return compactMapParts(nodes, edges, routes);
  }

  function compactMapParts(nodes, edges, routes) {
    var normalizedEdges = edges.length ? edges : edgesFromRoutes(routes);
    var nodeIds = new Set();
    normalizedEdges.forEach(function (edge) {
      nodeIds.add(edgeFrom(edge));
      nodeIds.add(edgeTo(edge));
    });

    var normalizedNodes = nodes.filter(function (node, index) {
      return !nodeIds.size || nodeIds.has(nodeIdentity(node, index));
    });
    if (!normalizedNodes.length) {
      normalizedNodes = nodesFromRoutes(routes);
    }

    return {
      nodes: dedupeById(normalizedNodes, nodeIdentity),
      edges: dedupeById(normalizedEdges, edgeIdentity)
    };
  }

  function addArrowMarker(svg) {
    var defs = svgElement("defs");
    var marker = svgElement("marker");
    marker.setAttribute("id", "arrowhead");
    marker.setAttribute("viewBox", "0 0 10 10");
    marker.setAttribute("refX", "8");
    marker.setAttribute("refY", "5");
    marker.setAttribute("markerWidth", "5");
    marker.setAttribute("markerHeight", "5");
    marker.setAttribute("orient", "auto-start-reverse");
    var path = svgElement("path");
    path.setAttribute("d", "M 0 0 L 10 5 L 0 10 z");
    path.setAttribute("fill", "currentColor");
    marker.appendChild(path);
    defs.appendChild(marker);
    svg.appendChild(defs);
  }

  function renderGroupLabel(group) {
    if (!group.label) {
      return;
    }
    var text = svgElement("text");
    text.setAttribute("class", "map-edge-label");
    text.setAttribute("x", "56");
    text.setAttribute("y", String(group.y));
    text.textContent = group.label;
    elements.map.appendChild(text);
  }

  function renderNode(node, index, positions) {
    var nodeId = nodeIdentity(node, index);
    var position = positions.get(nodeId);
    if (!position) {
      return;
    }
    var group = svgElement("g");
    group.setAttribute("class", "map-node");
    group.setAttribute("transform", "translate(" + position.x + " " + position.y + ")");
    applyProviderStyle(group, nodeProviderName(node));

    var rect = svgElement("rect");
    rect.setAttribute("rx", "16");
    rect.setAttribute("width", "190");
    rect.setAttribute("height", "58");
    group.appendChild(rect);

    var label = svgElement("text");
    label.setAttribute("x", "16");
    label.setAttribute("y", "25");
    label.textContent = nodeLabel(node);
    group.appendChild(label);

    var meta = svgElement("text");
    meta.setAttribute("x", "16");
    meta.setAttribute("y", "44");
    meta.textContent = nodeMeta(node);
    group.appendChild(meta);

    elements.map.appendChild(group);
  }

  function renderEdge(edge, index, positions) {
    var from = positions.get(edgeFrom(edge));
    var to = positions.get(edgeTo(edge));
    if (!from || !to) {
      return;
    }

    var edgeId = edgeIdentity(edge, index);
    var routeId = edgeRouteId(edge);
    var provider = edgeProviderName(edge);
    var startX = from.x + 190;
    var startY = from.y + 29;
    var endX = to.x;
    var endY = to.y + 29;
    var midX = startX + Math.max(80, (endX - startX) / 2);
    var pathData = "M " + startX + " " + startY + " C " + midX + " " + startY + ", " + (midX - 40) + " " + endY + ", " + endX + " " + endY;

    var group = svgElement("g");
    group.setAttribute("class", "map-edge-group");
    group.dataset.edgeId = edgeId;
    if (routeId) {
      group.dataset.routeSelect = routeId;
    }
    group.setAttribute("tabindex", "0");
    group.setAttribute("role", "button");
    group.setAttribute("aria-label", edgeLabel(edge, index));
    group.setAttribute("aria-current", String(edgeId === state.selectedEdgeId || routeId === state.selectedRouteId));
    applyProviderStyle(group, provider);

    var path = svgElement("path");
    path.setAttribute("class", "map-edge");
    path.setAttribute("d", pathData);
    group.appendChild(path);

    var hit = svgElement("path");
    hit.setAttribute("class", "map-edge-hit");
    hit.setAttribute("d", pathData);
    group.appendChild(hit);

    var label = svgElement("text");
    label.setAttribute("class", "map-edge-label");
    label.setAttribute("x", String((startX + endX) / 2 - 40));
    label.setAttribute("y", String((startY + endY) / 2 - 8));
    label.textContent = edgeText(edge);
    group.appendChild(label);

    group.addEventListener("click", function () {
      state.selectedEdgeId = edgeId;
      if (routeId) {
        selectRoute(routeId);
      } else {
        render();
      }
    });
    group.addEventListener("keydown", function (event) {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        state.selectedEdgeId = edgeId;
        if (routeId) {
          selectRoute(routeId);
        } else {
          render();
        }
      }
    });

    elements.map.appendChild(group);
  }

  function renderInspector(route) {
    clearElement(elements.inspector);
    if (!route) {
      elements.inspector.appendChild(paragraph("muted", "Select a model under an endpoint to inspect details."));
      return;
    }

    var heading = document.createElement("h3");
    heading.textContent = sourceModelName(route) + " → " + endpointLabel(runtimeFormat(route));
    elements.inspector.appendChild(heading);

    var chipRow = document.createElement("div");
    chipRow.className = "chip-row";
    routeChips(route).forEach(function (routeChip) {
      chipRow.appendChild(chip(routeChip.label, routeChip.kind));
    });
    elements.inspector.appendChild(chipRow);

    var list = document.createElement("dl");
    list.className = "definition-list";
    addDefinition(list, "Origin", isHotRoute(route) ? "User-defined adapter" : "Catalog route");
    addDefinition(list, "Route ID", routeIdentity(route, 0));
    addDefinition(list, "Source model", sourceModelName(route));
    addDefinition(list, "Runtime format", runtimeFormat(route));
    addDefinition(list, "Target provider", targetProviderName(route));
    addDefinition(list, "Target model", targetModelName(route));
    addDefinition(list, "Target format", targetFormat(route));
    addDefinition(list, "Config index", routeConfigIndex(route) == null ? "catalog" : routeConfigIndex(route));
    elements.inspector.appendChild(list);
  }

  function renderDiagnostics(graph) {
    clearElement(elements.diagnostics);
    var items = diagnostics(graph);

    if (!items.length) {
      elements.diagnostics.appendChild(paragraph("muted", "No diagnostics reported."));
      return;
    }

    items.forEach(function (item) {
      var severity = diagnosticSeverity(item);
      var diagnostic = document.createElement("article");
      diagnostic.className = "diagnostic diagnostic--" + classToken(severity);
      var code = document.createElement("p");
      code.className = "diagnostic__code";
      code.textContent = stringValue(item.code || item.id || item.kind, severity);
      var message = document.createElement("p");
      message.textContent = stringValue(item.message || item.detail || item.title, formatJson(item));
      diagnostic.appendChild(code);
      diagnostic.appendChild(message);
      elements.diagnostics.appendChild(diagnostic);
    });
  }

  function renderDraftStatus(graph) {
    var status = draftStatusText(graph);
    if (status && !state.saveBlockedReason && !state.draftNeedsValidation) {
      setEditorStatus(status);
    }
  }

  function renderTypedEditor() {
    if (!elements.typedRouteList) {
      return;
    }

    var routes = draftRoutes();
    clearElement(elements.typedRouteList);
    clampSelectedDraftIndex();

    if (!routes.length) {
      elements.typedRouteList.appendChild(paragraph("muted", "No draft source mappings. Create one to add an adapter."));
    }

    routes.forEach(function (route, index) {
      var button = document.createElement("button");
      button.type = "button";
      button.className = "route-list-item";
      button.dataset.draftRouteIndex = String(index);
      button.setAttribute("role", "option");
      button.setAttribute("aria-selected", String(index === state.selectedDraftRouteIndex));
      button.setAttribute("aria-current", String(index === state.selectedDraftRouteIndex));
      applyProviderStyle(button, route.target.provider);
      button.appendChild(strong(route.source.model || "New source"));
      button.appendChild(span("subtle", (route.source.format || "*") + " → " + route.target.provider + " / " + (route.target.model || "New target")));
      var chipRow = document.createElement("span");
      chipRow.className = "chip-row";
      chipRow.appendChild(chip(route.enabled === false ? "disabled" : "enabled", route.enabled === false ? "disabled" : "count"));
      button.appendChild(chipRow);
      elements.typedRouteList.appendChild(button);
    });

    var selected = routes[state.selectedDraftRouteIndex] || null;
    setTypedFormRoute(selected);
    updateActionButtons();
  }

  function setTypedFormRoute(route) {
    var hasRoute = !!route;
    [
      elements.sourceModelInput,
      elements.sourceFormatInput,
      elements.targetProviderInput,
      elements.targetModelInput,
      elements.targetFormatInput,
      elements.enabledInput,
      elements.moveUpButton,
      elements.moveDownButton,
      elements.enableButton,
      elements.disableButton,
      elements.updateRouteButton,
      elements.previewRouteButton,
      elements.routeValidateButton
    ].forEach(function (control) {
      if (control) {
        control.disabled = !hasRoute || state.isBusy;
      }
    });

    if (!hasRoute) {
      elements.sourceModelInput.value = "";
      elements.sourceFormatInput.value = "";
      elements.targetProviderInput.value = "codex";
      elements.targetModelInput.value = "";
      elements.targetFormatInput.value = "";
      elements.enabledInput.checked = true;
      return;
    }

    elements.sourceModelInput.value = route.source.model || "";
    elements.sourceFormatInput.value = route.source.format || "";
    elements.targetProviderInput.value = route.target.provider || "codex";
    elements.targetModelInput.value = route.target.model || "";
    elements.targetFormatInput.value = route.target.format || "";
    elements.enabledInput.checked = route.enabled !== false;
  }

  function handleManualRouteUpdate() {
    applyTypedFormToDraft();
    setEditorStatus("Selected route updated. Validate to refresh the projection.");
  }

  function applyTypedFormToDraft() {
    var routes = draftRoutes();
    var route = routes[state.selectedDraftRouteIndex];
    if (!route) {
      return;
    }

    route.source.model = elements.sourceModelInput.value.trim();
    setOptionalValue(route.source, "format", elements.sourceFormatInput.value);
    route.target.provider = elements.targetProviderInput.value;
    route.target.model = elements.targetModelInput.value.trim();
    setOptionalValue(route.target, "format", elements.targetFormatInput.value);
    route.enabled = elements.enabledInput.checked;
    updateDraftFromTypedEdit("Typed route changed. Validate to refresh the projection.");
  }

  function createDraftRoute() {
    var routes = draftRoutes();
    routes.push({
      source: { model: "" },
      target: { provider: "codex", model: "" },
      enabled: true
    });
    state.selectedDraftRouteIndex = routes.length - 1;
    updateDraftFromTypedEdit("New route added. Fill required fields, then validate.");
    focusRouteEditor();
  }

  function selectDraftRoute(index) {
    var routes = draftRoutes();
    if (!routes.length) {
      state.selectedDraftRouteIndex = 0;
      renderTypedEditor();
      return;
    }
    state.selectedDraftRouteIndex = Math.max(0, Math.min(index, routes.length - 1));
    renderTypedEditor();
    if (elements.sourceModelInput) {
      elements.sourceModelInput.focus({ preventScroll: true });
    }
  }

  function moveDraftRoute(direction) {
    var routes = draftRoutes();
    var from = state.selectedDraftRouteIndex;
    var to = from + direction;
    if (to < 0 || to >= routes.length) {
      return;
    }
    var route = routes.splice(from, 1)[0];
    routes.splice(to, 0, route);
    state.selectedDraftRouteIndex = to;
    updateDraftFromTypedEdit("Route order changed. Validate to refresh the projection.");
  }

  function setDraftRouteEnabled(enabled) {
    var route = draftRoutes()[state.selectedDraftRouteIndex];
    if (!route) {
      return;
    }
    route.enabled = enabled;
    updateDraftFromTypedEdit(enabled ? "Route enabled. Validate to refresh the projection." : "Route disabled. Validate to refresh the projection.");
  }

  function updateDraftFromTypedEdit(message) {
    state.draftConfig = normalizeConfig(state.draftConfig);
    state.draftNeedsValidation = true;
    state.saveBlockedReason = "Validate the draft before saving.";
    syncDraftPreview();
    renderTypedEditor();
    setEditorStatus(message);
    updateActionButtons();
  }

  function selectRoute(routeId) {
    state.selectedRouteId = routeId;
    state.selectedEdgeId = edgeIdForRoute(state.currentGraph, routeId);
    state.selectedDraftRouteIndex = selectedIndexFromRouteId(state.currentGraph, routeId);
    render();
  }

  function selectAdjacentRoute(direction) {
    var routes = focalRoutes(state.currentGraph || {});
    if (!routes.length) {
      return;
    }
    var currentIndex = routes.findIndex(function (route, index) {
      return routeIdentity(route, index) === state.selectedRouteId;
    });
    var nextIndex = currentIndex < 0 ? 0 : Math.max(0, Math.min(currentIndex + direction, routes.length - 1));
    selectRoute(routeIdentity(routes[nextIndex], nextIndex));
  }

  function selectedRoute(routes) {
    return routes.find(function (route, index) {
      return routeIdentity(route, index) === state.selectedRouteId;
    }) || null;
  }

  function edgeIdForRoute(graph, routeId) {
    var edges = graphEdges(graph || {});
    for (var index = 0; index < edges.length; index += 1) {
      if (edgeRouteId(edges[index]) === routeId) {
        return edgeIdentity(edges[index], index);
      }
    }
    return null;
  }

  function selectedIndexFromRouteId(graph, routeId) {
    var routes = effectiveRoutes(graph || {}).concat(routeCards(graph || {}));
    for (var index = 0; index < routes.length; index += 1) {
      if (routeIdentity(routes[index], index) === routeId && routeConfigIndex(routes[index]) != null) {
        return routeConfigIndex(routes[index]);
      }
    }
    return Math.min(state.selectedDraftRouteIndex || 0, Math.max(draftRoutes().length - 1, 0));
  }

  function routeIdentity(route, index) {
    return stringValue(route.route_id || route.id || route.row_id || route.effective_route_id || route.key, "route:" + index);
  }

  function routeConfigIndex(route) {
    var value = route.config_index;
    if (value == null && route.configIndex != null) {
      value = route.configIndex;
    }
    if (typeof value === "number") {
      return value;
    }
    if (typeof value === "string" && value !== "" && !Number.isNaN(Number(value))) {
      return Number(value);
    }
    return null;
  }

  function sourceModelName(route) {
    return nestedValue(route, ["source", "model"], route.source_model || route.model || route.sourceModel || "unknown source");
  }

  function runtimeFormat(route) {
    return stringValue(
      route.runtime_format ||
      route.source_runtime_format ||
      route.sourceFormat ||
      nestedValue(route, ["source", "runtime_format"], null) ||
      nestedValue(route, ["source", "format"], null),
      "*"
    );
  }

  function targetProviderName(route) {
    return stringValue(nestedValue(route, ["target", "provider"], route.target_provider || route.provider || route.targetProvider), "unknown provider");
  }

  function targetModelName(route) {
    return stringValue(nestedValue(route, ["target", "model"], route.target_model || route.targetModel), "unknown target");
  }

  function targetFormat(route) {
    return stringValue(
      nestedValue(
        route,
        ["target", "format"],
        nestedValue(route, ["target", "provider_format"], route.target_provider_format || route.target_format || route.targetFormat)
      ),
      "provider default"
    );
  }

  function routeLabel(route, index) {
    return routeIdentity(route, index) + " · " + sourceKind(route);
  }

  function sourceKind(route) {
    if (isHotRoute(route)) {
      return "adapter";
    }
    return "catalog";
  }

  function isHotRoute(route) {
    var source = stringValue(route.source_type || route.source || route.origin || route.kind || route.route_type, "").toLowerCase();
    return source.indexOf("hot") !== -1 || route.mutable === true || routeConfigIndex(route) != null;
  }

  function routeChips(route) {
    var chips = [];
    chips.push(isHotRoute(route) ? { label: "adapter", kind: "count" } : { label: "catalog", kind: "catalog" });

    if (route.enabled === false || route.disabled === true) {
      chips.push({ label: "disabled", kind: "disabled" });
    }

    if (route.shadowed === true || route.state === "shadowed") {
      chips.push({ label: "shadowed", kind: "shadowed" });
    }
    if (route.active === false || route.state === "inactive_source_format" || route.state === "inactive") {
      chips.push({ label: "inactive", kind: "inactive" });
    }
    if (route.valid === false || route.state === "invalid") {
      chips.push({ label: "invalid", kind: "invalid" });
    }
    if (runtimeFormat(route) !== "*") {
      chips.push({ label: "format-specific", kind: "format" });
    }
    return chips;
  }

  function providerNames(graph, routes) {
    var names = new Set();
    routes.forEach(function (route) {
      names.add(targetProviderName(route));
    });
    graphNodes(graph).forEach(function (node) {
      var provider = nodeProviderName(node);
      if (provider) {
        names.add(provider);
      }
    });
    graphEdges(graph).forEach(function (edge) {
      var provider = edgeProviderName(edge);
      if (provider) {
        names.add(provider);
      }
    });
    return Array.from(names).filter(Boolean).sort();
  }

  function assignProviderStyles(graph, routes) {
    providerNames(graph, routes).forEach(function (provider) {
      providerStyle(provider);
    });
  }

  function providerStyle(provider) {
    var name = provider || "unknown provider";
    if (!state.providerStyleByName.has(name)) {
      var palette = providerStyles[state.providerStyleByName.size % providerStyles.length];
      state.providerStyleByName.set(name, palette);
    }
    return state.providerStyleByName.get(name);
  }

  function applyProviderStyle(element, provider) {
    var palette = providerStyle(provider);
    element.classList.add(palette.className);
  }

  function layoutNodes(nodes, groups) {
    var sourceNodes = [];
    var targetNodes = [];
    nodes.forEach(function (node, index) {
      if (nodeRole(node) === "target") {
        targetNodes.push({ node: node, index: index });
      } else {
        sourceNodes.push({ node: node, index: index });
      }
    });

    if (!targetNodes.length) {
      var split = Math.ceil(nodes.length / 2);
      sourceNodes = nodes.slice(0, split).map(function (node, index) { return { node: node, index: index }; });
      targetNodes = nodes.slice(split).map(function (node, index) { return { node: node, index: index + split }; });
    }

    var positions = new Map();
    var groupLabels = [];
    var rowGap = 86;
    var sourceOffset = groupLabelOffset(groups);
    sourceNodes.forEach(function (entry, row) {
      positions.set(nodeIdentity(entry.node, entry.index), { x: 56, y: 54 + sourceOffset + row * rowGap });
    });
    targetNodes.forEach(function (entry, row) {
      positions.set(nodeIdentity(entry.node, entry.index), { x: 520, y: 54 + sourceOffset + row * rowGap });
    });

    graphGroupsFromLayout(groups).forEach(function (group, index) {
      groupLabels.push({
        label: group,
        y: 28 + index * 28
      });
    });

    return {
      positions: positions,
      groups: groupLabels,
      width: 780,
      height: 120 + sourceOffset + Math.max(sourceNodes.length, targetNodes.length, 2) * rowGap
    };
  }

  function graphGroupsFromLayout(groups) {
    return (groups || []).map(function (group) {
      return stringValue(group.title || group.label || group.name || group.id, "");
    }).filter(Boolean).slice(0, 4);
  }

  function groupLabelOffset(groups) {
    return graphGroupsFromLayout(groups).length ? graphGroupsFromLayout(groups).length * 24 : 0;
  }

  function nodesFromRoutes(routes) {
    var byId = new Map();
    routes.forEach(function (route, index) {
      var sourceId = "source:" + sourceModelName(route) + ":" + runtimeFormat(route);
      var targetId = "target:" + targetProviderName(route) + ":" + targetModelName(route);
      if (!byId.has(sourceId)) {
        byId.set(sourceId, { id: sourceId, role: "source", label: sourceModelName(route), format: runtimeFormat(route) });
      }
      if (!byId.has(targetId)) {
        byId.set(targetId, { id: targetId, role: "target", provider: targetProviderName(route), label: targetModelName(route), format: targetFormat(route) });
      }
      route.__fallback_edge = "edge:" + index;
    });
    return Array.from(byId.values());
  }

  function edgesFromRoutes(routes) {
    return routes.map(function (route, index) {
      return {
        id: "edge:" + index,
        route_id: routeIdentity(route, index),
        from: "source:" + sourceModelName(route) + ":" + runtimeFormat(route),
        to: "target:" + targetProviderName(route) + ":" + targetModelName(route),
        runtime_format: runtimeFormat(route),
        target_provider: targetProviderName(route)
      };
    });
  }

  function nodeIdentity(node, index) {
    return stringValue(node.id || node.node_id || node.key, "node:" + index);
  }

  function nodeRole(node) {
    var value = stringValue(node.role || node.kind || node.type, "").toLowerCase();
    if (value.indexOf("target") !== -1) {
      return "target";
    }
    return "source";
  }

  function nodeProviderName(node) {
    return stringValue(node.provider || node.target_provider, "");
  }

  function nodeLabel(node) {
    return stringValue(node.label || node.model || node.name || node.id, "node");
  }

  function nodeMeta(node) {
    return stringValue(node.format || node.runtime_format || node.provider || node.kind, "");
  }

  function edgeIdentity(edge, index) {
    return stringValue(edge.id || edge.edge_id || edge.key, "edge:" + index);
  }

  function edgeRouteId(edge) {
    return stringValue(edge.route_id || edge.effective_route_id || edge.row_id, "");
  }

  function edgeFrom(edge) {
    return stringValue(edge.from || edge.source || edge.source_id || edge.from_node, "");
  }

  function edgeTo(edge) {
    return stringValue(edge.to || edge.target || edge.target_id || edge.to_node, "");
  }

  function edgeProviderName(edge) {
    return stringValue(edge.target_provider || edge.provider || nestedValue(edge, ["target", "provider"], ""), "");
  }

  function edgeText(edge) {
    return stringValue(edge.runtime_format || edge.source_runtime_format || edge.format || edge.label, "route");
  }

  function edgeLabel(edge, index) {
    return "Route edge " + edgeIdentity(edge, index) + " " + edgeText(edge);
  }

  function draftRoutes() {
    state.draftConfig = normalizeConfig(state.draftConfig || { routes: [] });
    return state.draftConfig.routes;
  }

  function normalizeConfig(config) {
    var normalized = config && typeof config === "object" && !Array.isArray(config) ? cloneConfig(config) : { routes: [] };
    if (!Array.isArray(normalized.routes)) {
      normalized.routes = [];
    }
    normalized.routes = normalized.routes.map(normalizeDraftRoute);
    return normalized;
  }

  function normalizeDraftRoute(route) {
    route = route || {};
    var source = route && typeof route.source === "object" ? cloneConfig(route.source) : {};
    var target = route && typeof route.target === "object" ? cloneConfig(route.target) : {};
    source.model = stringValue(source.model || route.source_model || route.model, "");
    setOptionalValue(source, "format", source.format || route.source_runtime_format || route.runtime_format);
    target.provider = stringValue(target.provider || route.target_provider || route.provider, "codex");
    target.model = stringValue(target.model || route.target_model, "");
    setOptionalValue(target, "format", target.format || route.target_provider_format || route.target_format);
    return {
      source: source,
      target: target,
      enabled: route.enabled !== false
    };
  }

  function setOptionalValue(object, key, value) {
    var text = stringValue(value, "").trim();
    if (text) {
      object[key] = text;
    } else {
      delete object[key];
    }
  }

  function stableDraftText() {
    return formatJson(state.draftConfig || { routes: [] });
  }

  function cloneConfig(value) {
    return JSON.parse(JSON.stringify(value == null ? {} : value));
  }

  function clampSelectedDraftIndex() {
    var routes = draftRoutes();
    if (!routes.length) {
      state.selectedDraftRouteIndex = 0;
      return;
    }
    state.selectedDraftRouteIndex = Math.max(0, Math.min(state.selectedDraftRouteIndex || 0, routes.length - 1));
  }

  function blockingReason(graph) {
    var blocking = diagnostics(graph || {}).filter(isBlockingDiagnostic);
    if (blocking.length) {
      return "Resolve " + blocking.length + " blocking " + pluralize(blocking.length, "diagnostic") + " before saving.";
    }

    var status = (graph || {}).draft_status;
    if (draftStatusBlocks(status)) {
      if (typeof status === "string") {
        return "Draft is " + status.replace(/_/g, " ") + ".";
      }
      return stringValue(status.message || status.reason, "Draft is blocked by validation status.");
    }

    return "";
  }

  function isBlockingDiagnostic(item) {
    var severity = diagnosticSeverity(item);
    return item.blocking === true || severity === "error" || severity === "blocking" || severity === "fatal";
  }

  function diagnosticSeverity(item) {
    return stringValue(item.severity || item.level || item.status || (item.blocking ? "blocking" : "info"), "info").toLowerCase();
  }

  function draftStatusBlocks(status) {
    if (typeof status === "string") {
      var statusText = status.toLowerCase();
      return statusText !== "" && statusText !== "valid";
    }
    if (!status || typeof status !== "object") {
      return false;
    }
    if (status.blocking === true) {
      return true;
    }
    if (Array.isArray(status.blockers) && status.blockers.length) {
      return true;
    }
    if (Array.isArray(status.diagnostics) && status.diagnostics.some(isBlockingDiagnostic)) {
      return true;
    }
    var value = stringValue(status.status || status.state || status.kind, "").toLowerCase();
    return value === "blocked" || value === "invalid" || value === "error";
  }

  function draftStatusText(graph) {
    var status = graph && graph.draft_status;
    if (typeof status === "string" && status) {
      return "Draft status: " + status.replace(/_/g, " ") + ".";
    }
    if (!status || typeof status !== "object") {
      return "";
    }
    return stringValue(status.message || status.summary || status.status || status.state, "");
  }

  function updateActionButtons() {
    var routes = draftRoutes();
    var hasSelectedRoute = routes.length > 0;
    state.saveBlockedReason = state.isBusy ? state.saveBlockedReason : (state.draftNeedsValidation ? "Validate the draft before saving." : blockingReason(state.currentGraph));

    [elements.validateDraft, elements.routeValidateButton, elements.previewRouteButton, elements.revertConfig, elements.newRouteButton].forEach(function (button) {
      if (button) {
        button.disabled = state.isBusy;
      }
    });

    [elements.saveConfig, elements.routeSaveButton].forEach(function (button) {
      if (button) {
        button.disabled = state.isBusy || !!state.saveBlockedReason;
        button.title = state.saveBlockedReason || "";
      }
    });

    [elements.sourceModelInput, elements.sourceFormatInput, elements.targetProviderInput, elements.targetModelInput, elements.targetFormatInput, elements.enabledInput].forEach(function (control) {
      if (control) {
        control.disabled = state.isBusy || !hasSelectedRoute;
      }
    });

    if (elements.moveUpButton) {
      elements.moveUpButton.disabled = state.isBusy || !hasSelectedRoute || state.selectedDraftRouteIndex <= 0;
    }
    if (elements.moveDownButton) {
      elements.moveDownButton.disabled = state.isBusy || !hasSelectedRoute || state.selectedDraftRouteIndex >= routes.length - 1;
    }
    if (elements.toggleEnabledButton) {
      elements.toggleEnabledButton.disabled = state.isBusy || !hasSelectedRoute;
    }
    if (elements.updateRouteButton) {
      elements.updateRouteButton.disabled = state.isBusy || !hasSelectedRoute;
    }
    if (elements.enableButton) {
      elements.enableButton.disabled = state.isBusy || !hasSelectedRoute || (draftRoutes()[state.selectedDraftRouteIndex] || {}).enabled !== false;
    }
    if (elements.disableButton) {
      elements.disableButton.disabled = state.isBusy || !hasSelectedRoute || (draftRoutes()[state.selectedDraftRouteIndex] || {}).enabled === false;
    }
  }

  function clearElement(element) {
    while (element.firstChild) {
      element.removeChild(element.firstChild);
    }
  }

  function chip(label, kind) {
    var node = document.createElement("span");
    node.className = "chip chip--" + classToken(kind || label);
    node.textContent = label;
    return node;
  }

  function strong(value) {
    var node = document.createElement("strong");
    node.textContent = stringValue(value, "");
    return node;
  }

  function span(className, value) {
    var node = document.createElement("span");
    node.className = className;
    node.textContent = stringValue(value, "");
    return node;
  }

  function paragraph(className, value) {
    var node = document.createElement("p");
    node.className = className;
    node.textContent = value;
    return node;
  }

  function addDefinition(list, term, value) {
    var wrapper = document.createElement("div");
    var dt = document.createElement("dt");
    var dd = document.createElement("dd");
    dt.textContent = term;
    dd.textContent = stringValue(value, "");
    wrapper.appendChild(dt);
    wrapper.appendChild(dd);
    list.appendChild(wrapper);
  }

  function svgElement(name) {
    return document.createElementNS(SVG_NS, name);
  }

  function dedupeById(items, identity) {
    var seen = new Set();
    var output = [];
    items.forEach(function (item, index) {
      var id = identity(item, index);
      if (!seen.has(id)) {
        seen.add(id);
        output.push(item);
      }
    });
    return output;
  }

  function nestedValue(object, path, fallback) {
    var current = object;
    for (var index = 0; index < path.length; index += 1) {
      if (!current || typeof current !== "object" || !(path[index] in current)) {
        return fallback;
      }
      current = current[path[index]];
    }
    return current == null ? fallback : current;
  }

  function stringValue(value, fallback) {
    if (value == null || value === "") {
      return fallback;
    }
    return String(value);
  }

  function classToken(value) {
    return stringValue(value, "item").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "item";
  }

  function formatJson(value) {
    return JSON.stringify(value, null, 2);
  }

  function pluralize(count, noun) {
    return count === 1 ? noun : noun + "s";
  }

  function prefersReducedMotion() {
    return window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  }

  function setBusy(isBusy, message) {
    state.isBusy = isBusy;
    updateActionButtons();
    if (message) {
      elements.statusText.textContent = message;
    }
    elements.statusDot.dataset.state = isBusy ? "busy" : "ready";
  }

  function setReady(message) {
    elements.statusDot.dataset.state = "ready";
    elements.statusText.textContent = message;
  }

  function showError(error) {
    elements.statusDot.dataset.state = "error";
    elements.statusText.textContent = "Config route map needs attention.";
    elements.errorState.hidden = false;
    elements.errorState.textContent = error.message || String(error);
  }

  function clearError() {
    elements.errorState.hidden = true;
    elements.errorState.textContent = "";
  }

  function setEditorStatus(message) {
    if (elements.editorStatus) {
      elements.editorStatus.textContent = message;
    }
    if (elements.routeEditorStatus) {
      elements.routeEditorStatus.textContent = message;
    }
  }
})();
