-- Your SQL goes here

-- Create function to update mean
CREATE FUNCTION update_mean_on_del() RETURNS TRIGGER AS
$BODY$
BEGIN
    IF (SELECT
            CASE score_number WHEN 1 THEN true 
                              WHEN 0 THEN true
                              ELSE false
            END
        FROM means 
        WHERE means.user_id = old.user_id) THEN

        DELETE FROM means 
        WHERE means.user_id = old.user_id;
    ELSE 
        UPDATE means 
        SET val = ((means.val * means.score_number) - old.score) / (means.score_number - 1), score_number = means.score_number 
        - 1;
    END IF;

    RETURN old;
END;
$BODY$
LANGUAGE plpgsql;

-- Create the trigger on insert
CREATE TRIGGER update_means_on_del_rating
AFTER DELETE ON ratings
FOR EACH ROW
EXECUTE FUNCTION update_mean_on_del();