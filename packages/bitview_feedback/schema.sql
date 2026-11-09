-- The inbox: messages not yet read by the maintainer's job, which deletes each row as it takes it.
CREATE TABLE feedback (
  id      INTEGER PRIMARY KEY AUTOINCREMENT, -- never reused: the job keeps a message under its id
  day     TEXT NOT NULL,        -- UTC date, no time of day
  page    TEXT NOT NULL,        -- the app's folder, as /compass/
  link    TEXT NOT NULL,        -- the exact address, masked, '' if the sender unticked it
  version TEXT NOT NULL,        -- the page's content hash (Bitview-Version), or ''
  browser TEXT NOT NULL,        -- chromium|firefox|safari|other, -, phone|desk
  context TEXT NOT NULL,        -- JSON: build, os, browser major, viewport (10 px), scale, theme, mode, recent errors
  message TEXT NOT NULL,
  contact TEXT NOT NULL,        -- '' or what the sender chose to give
  UNIQUE (page, message)        -- an exact repeat while unread is one row
);
