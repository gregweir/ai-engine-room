<script lang="ts">
  let { preview }: { preview: string } = $props();
</script>

<section class="report-preview" aria-labelledby="report-preview-heading">
  <h3 id="report-preview-heading">Report-safe preview</h3>
  <p class="report-note">
    This is the report-safe projection only — deliberately narrower than the
    full state above. It excludes raw errors, internal ids, and anything not
    approved for sharing.
  </p>
  <pre class="report-preview-text">{preview}</pre>
</section>
