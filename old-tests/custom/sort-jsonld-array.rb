#!/usr/bin/env ruby
# This is a small utility script to sort
# the array of JSON-LD objects outputed by
# the expansion algorithm on tests 0122 to 0125.
# The result can then be compared to the expected
# expanded output using a JSON comparison tool such as
# http://jsondiff.com/
require 'json'

array = JSON.parse(STDIN.read)
array.sort_by! { |obj| obj['@id'] }
puts JSON.pretty_generate(array)