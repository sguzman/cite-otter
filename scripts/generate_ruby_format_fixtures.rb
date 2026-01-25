#!/usr/bin/env ruby
# frozen_string_literal: true

require 'fileutils'
require 'json'

root = File.expand_path('..', __dir__)
anystyle_lib = File.join(root, 'tmp', 'anystyle', 'lib')
$LOAD_PATH.unshift(anystyle_lib)

begin
  require 'anystyle'
rescue LoadError => e
  warn "failed to load anystyle from #{anystyle_lib}: #{e.message}"
  exit 1
end

REPORT_DIR = File.join(root, 'target', 'reports', 'ruby-format')
FileUtils.mkdir_p(REPORT_DIR)

def read_refs(path)
  File.read(path).lines.map(&:strip).reject(&:empty?)
end

def write_csl(path, refs)
  entries = AnyStyle.parse(refs, format: :csl)
  output = entries.map { |entry| JSON.generate(entry) }.join("\n")
  File.write(path, output)
end

def write_bibtex(path, refs)
  bibliography = AnyStyle.parse(refs, format: :bibtex)
  File.write(path, bibliography.to_s)
end

core_refs = read_refs(File.join(root, 'tests', 'fixtures', 'format', 'core-refs.txt'))
sample_refs = read_refs(File.join(root, 'tests', 'fixtures', 'format', 'refs.txt'))

write_csl(File.join(REPORT_DIR, 'core-csl.txt'), core_refs)
write_bibtex(File.join(REPORT_DIR, 'core-bibtex.txt'), core_refs)
write_csl(File.join(REPORT_DIR, 'csl.txt'), sample_refs)
write_bibtex(File.join(REPORT_DIR, 'bibtex.txt'), sample_refs)

puts "ruby format fixtures written to #{REPORT_DIR}"
