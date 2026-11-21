## #3181 - Record the control installed on a machine

**Decision.** Store the installed control model and exact NC software identifier
in the machine file and in saved projects. The machine can also state its
installed options and tool-change procedure. An omitted option list means
unknown. An empty list means the installation has no options. The Machines
pane shows the stated facts and the post engine exposes them to scripts.
Saved projects use format version 75. A version 74 project opens with
these facts unstated. A version 74 reader refuses the newer file so the
next save cannot silently discard the machine's installation facts.

**Reason.** One controller family spans several releases. Each machine can
have different options, a tool changer, or a manual change procedure. The post
owns its supported release list and the syntax of each operation. A newer
release number alone gives no evidence of compatible output.

**Alternatives.** A controller profile per post in the machine file would tie
machine records to one output language. A flag per controller feature would
duplicate the post's syntax rules. Future profiles can share a post by adding
the documented release and an acceptance test. Existing files can acquire
installation facts without changing their chosen post.

**First installation.** The Picomax 55 uses the iTNC 530, NC software
340490-03, in three-axis CNC mode with W fixed before establishing the
preset. The inventory comes from the installation's 14108131.MP parameters
and the Fehlmann manuals 141-49-07-E and P07-35-06-E. The machine file omits
absolute travel bands because their reference frame needs confirmation on
the machine. It also leaves the installed software options unstated.

**Evidence.** The earlier prototype's one-tool and two-tool programs reached
the end on the programming station. Its update history identifies release
`606425 004 SP20`. The converter accepted 41 and 63 blocks respectively.
These tests apply to those two programs. On the 530, Cycle 247 has no effect during Test Run. The maximum M140 retract can
end at the active travel limit or stop earlier under collision monitoring.
Checks on the physical machine still need to establish preset activation,
the upper change position, the drive-off procedure and coolant operation.
The [34049x-03 programming manual](https://content.heidenhain.de/doku/tnc_guide/pdf_files/iTNC530/34049x-neu/en/bhb.pdf)
describes the retract in section 7.4 (page 303) and the Test Run restriction
in section 8.9 (page 492). The maintainer approved separate machine and post
changes in [#3098](https://github.com/NeoTech/Camcraft/issues/3098).
