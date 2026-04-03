.[0]?
| .path as $p
| (.entities // [])
| map({
    description: ("PII: " + (.entity_type | tostring)),
    check_name: ("AEGIS/" + (.entity_type | tostring)),
    severity: "major",
    fingerprint: ($p + ":" + ((.start // 0) | tostring) + ":" + ((.end // 0) | tostring)),
    location: {
      path: $p,
      lines: { begin: 1, end: 1 }
    }
  })
